//! Kritéria přijetí B1 až B4 (`ARCHITECTURE.md` §12.4): tři nody proti lokálně spuštěnému Workeru (`wrangler dev`, mock Cloudflare).
//!
//! Test potřebuje nainstalované závislosti Workeru (`cd worker && npm install`); bez nich se přeskočí.
//! `ANCORD_SKIP_E2E=1` ho vypne.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};

use ancord_cli::online::{Online, OnlineEvent, Persist};
use ancord_cli::{
    address, parse_address, register_push, ack_inbox, admit, admit_unchecked, process_inbox, start_conversation, connect_any, fetch_route, lookup_profile, message_status, next_event, next_pushed, publish_profile, receive, rpc, send_typing, subscribe,
    subscribe_many, transmit, transmit_with_push, CliError, Event, Role, Route, RouteNode, Session,
};
use ancord_node::user::UserKeys;
use ancord_node::canonical::canonical_bytes;
use ancord_node::encoding::{b64u_decode_array, b64u_encode, base32_encode};
use ancord_node::identity::{hostname_of, Identity};
use ancord_node::sign::sign;
use ancord_node::time::unix_now;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Router;
use ed25519_dalek::SigningKey;
use futures_util::StreamExt;
use rand::rngs::OsRng;
use serde_json::{json, Value};
use tokio::process::{Child, Command};
use tokio_tungstenite::tungstenite::Message;

const ZONE: &str = "melonet.cz";

/// Online klient v testu nic neukládá.
struct NoDisk;

impl Persist for NoDisk {
    fn save_keys(&mut self, _: &UserKeys) -> Result<(), CliError> {
        Ok(())
    }
    fn save_conversation(&mut self, _: usize, _: &Session) -> Result<(), CliError> {
        Ok(())
    }
    fn add_conversation(&mut self, _: usize, _: &Session) -> Result<(), CliError> {
        Ok(())
    }
}

/// Další zpráva online klienta (signály psaní a prázdné rámce přeskočí).
async fn next_online_message(online: &mut Online, ws: &mut ancord_cli::NodeSocket) -> Vec<u8> {
    let until = Instant::now() + Duration::from_secs(10);
    while Instant::now() < until {
        for e in online.next(ws, &mut NoDisk, Duration::from_secs(1)).await.unwrap().unwrap_or_default() {
            if let OnlineEvent::Message { text, .. } = e {
                return text;
            }
        }
    }
    panic!("no message for the online client");
}
const ADMIN: &str = "e2e-admin";

fn worker_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../worker")
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

/// Proces i s potomky (wrangler spouští workerd): při zániku se zabije celá skupina.
struct Proc {
    child: Child,
}

impl Proc {
    fn spawn(mut cmd: Command, log: &Path) -> Self {
        let out = std::fs::File::create(log).unwrap();
        cmd.stdin(Stdio::null()).stdout(out.try_clone().unwrap()).stderr(out).process_group(0).kill_on_drop(true);
        Self { child: cmd.spawn().unwrap() }
    }
}

impl Drop for Proc {
    fn drop(&mut self) {
        if let Some(pid) = self.child.id() {
            let _ = std::process::Command::new("kill").args(["-KILL", "--", &format!("-{pid}")]).status();
        }
    }
}

struct NodeProc {
    id: String,
    hostname: String,
    port: u16,
    dir: PathBuf,
    proc: Option<Proc>,
}

fn spawn_node(n: &mut NodeProc, worker_port: u16, fwd_port: u16, log_dir: &Path, generation: u32) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ancord-node"));
    cmd.env("TRUST_WORKER_KEY_ON_FIRST_USE", "1").env("WORKER_URL", format!("http://127.0.0.1:{worker_port}"))
        .env("DATA_DIR", &n.dir)
        .env("LISTEN", format!("127.0.0.1:{}", n.port))
        .env("CLOUDFLARED_BIN", "none")
        .env("CLAIM_CPU_CORES", "2")
        .env("CLAIM_RAM_MB", "256")
        .env("CLAIM_STORAGE_GB", "1")
        .env("CLAIM_BANDWIDTH_MBPS", "100")
        .env("AUTH_TIMEOUT_S", "2")
        .env("PEER_URL_TEMPLATE", format!("http://127.0.0.1:{fwd_port}/{{host}}"))
        .env("PEER_TIMEOUT_S", "3")
        .env("RUST_LOG", "info");
    n.proc = Some(Proc::spawn(cmd, &log_dir.join(format!("{}-{generation}.log", &n.id[..8]))));
}

/// „Tunel“: `/{hostname}/{cesta}` se předá nodu na jeho lokálním portu (metoda, hlavičky i tělo zůstanou).
async fn forward(State(ports): State<Arc<HashMap<String, u16>>>, req: axum::extract::Request) -> Response {
    let (parts, body) = req.into_parts();
    let pq = parts.uri.path_and_query().map_or("", |p| p.as_str());
    let Some((host, tail)) = pq.trim_start_matches('/').split_once('/') else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let Some(port) = ports.get(host) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Ok(bytes) = axum::body::to_bytes(body, 1 << 20).await else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let mut out = reqwest::Client::new().request(parts.method, format!("http://127.0.0.1:{port}/{tail}")).body(bytes);
    if let Some(auth) = parts.headers.get("authorization") {
        out = out.header("authorization", auth);
    }
    match out.send().await {
        Ok(r) => {
            let status = StatusCode::from_u16(r.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
            (status, r.bytes().await.unwrap_or_default()).into_response()
        }
        Err(_) => StatusCode::BAD_GATEWAY.into_response(),
    }
}

async fn eventually<T, F, Fut>(what: &str, timeout: Duration, mut f: F) -> T
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Option<T>>,
{
    let end = Instant::now() + timeout;
    loop {
        if let Some(v) = f().await {
            return v;
        }
        assert!(Instant::now() < end, "timeout waiting for: {what}");
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

async fn admin_nodes(http: &reqwest::Client, base: &str) -> Vec<Value> {
    let res = http.get(format!("{base}/admin/nodes")).bearer_auth(ADMIN).send().await.unwrap();
    res.json().await.unwrap_or_default()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn three_nodes_register_challenge_route_and_admit() {
    let wrangler = worker_dir().join("node_modules/.bin/wrangler");
    if std::env::var_os("ANCORD_SKIP_E2E").is_some() || !wrangler.exists() {
        eprintln!("skipping e2e: run `npm install` in worker/ (or unset ANCORD_SKIP_E2E)");
        return;
    }
    let tmp = tempfile::tempdir().unwrap();
    let logs = tmp.path().join("logs");
    std::fs::create_dir_all(&logs).unwrap();

    // identity se vytvoří předem, aby šla zapsat do genesis allowlistu Workeru
    let mut nodes: Vec<NodeProc> = (0..3)
        .map(|i| {
            let dir = tmp.path().join(format!("node{i}"));
            let id = Identity::load_or_create(&dir).unwrap();
            NodeProc { hostname: hostname_of(id.node_id(), ZONE), id: id.node_id().to_owned(), port: free_port(), dir, proc: None }
        })
        .collect();

    // „tunel“: Worker žádá o důkaz na PROOF_ORIGIN/{hostname}, rozcestník to předá správnému nodu
    let ports: Arc<HashMap<String, u16>> = Arc::new(nodes.iter().map(|n| (n.hostname.clone(), n.port)).collect());
    let fwd_port = free_port();
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", fwd_port)).await.unwrap();
    let app = Router::new().fallback(forward).with_state(ports);
    let fwd = tokio::spawn(async move { axum::serve(listener, app).await });

    let worker_port = free_port();
    let genesis = nodes.iter().map(|n| n.id.as_str()).collect::<Vec<_>>().join(",");
    let overrides = r#"{"epochSeconds":15,"snapshotRefreshS":2,"challenges":{"cpu":{"iters":100000,"every":1000},"space":{"ram":{"maxLeaves":4096},"storage":{"maxLeaves":16384}}}}"#;
    let mut cmd = Command::new(&wrangler);
    cmd.current_dir(worker_dir())
        .args(["dev", "--port", &worker_port.to_string(), "--persist-to"])
        .arg(tmp.path().join("wrangler-state"))
        .env("WRANGLER_SEND_METRICS", "false")
        .env("CI", "1");
    for (k, v) in [
        ("WORKER_SIGNING_KEY", "AQIDBAUGBwgJCgsMDQ4PEBESExQVFhcYGRobHB0eHyA"),
        ("CHALLENGE_HMAC_KEY", "ISIjJCUmJygpKissLS4vMDEyMzQ1Njc4OTo7PD0-Pz8"),
        ("PUSH_ENC_KEY", "QEFCQ0RFRkdISUpLTE1OT1BRUlNUVVZXWFlaW1xdXl8"),
        ("ADMIN_TOKEN", ADMIN),
        ("GENESIS_NODE_IDS", genesis.as_str()),
        ("PROOF_ORIGIN", &format!("http://127.0.0.1:{fwd_port}/{{hostname}}")),
        ("CONFIG_OVERRIDES", overrides),
    ] {
        cmd.args(["--var", &format!("{k}:{v}")]);
    }
    let _wrangler = Proc::spawn(cmd, &logs.join("wrangler.log"));

    let base = format!("http://127.0.0.1:{worker_port}");
    let http = reqwest::Client::new();
    let ready = eventually("wrangler dev", Duration::from_secs(120), || async {
        http.get(format!("{base}/health")).send().await.ok().filter(|r| r.status().is_success())
    })
    .await;
    drop(ready);

    for n in &mut nodes {
        spawn_node(n, worker_port, fwd_port, &logs, 0);
    }

    // B1: všechny tři nody aktivní a s naměřenou kapacitou
    eventually("3 active nodes with measured capacity", Duration::from_secs(240), || async {
        let list = admin_nodes(&http, &base).await;
        let ok = list.len() == 3
            && list.iter().all(|n| n["status"] == "active" && n["live"] == true && n["challenges_ok"].as_u64().unwrap_or(0) >= 4 && n["strikes"] == 0);
        ok.then_some(())
    })
    .await;

    // adresář je v epoše neměnný: node, který v něm není, nesmí koordinovat zápis. Čeká se na epochu se všemi třemi.
    eventually("directory with all three nodes", Duration::from_secs(60), || async {
        let dir: Value = http.get(format!("{base}/directory")).send().await.ok()?.json().await.ok()?;
        (dir["body"]["nodes"].as_array().map(Vec::len) == Some(3)).then_some(())
    })
    .await;
    let epoch_with_all = http.get(format!("{base}/directory")).send().await.unwrap().json::<Value>().await.unwrap()["body"]["epoch"].as_u64().unwrap();
    for n in &nodes {
        let log_path = logs.join(format!("{}-0.log", &n.id[..8]));
        eventually("node holds that directory", Duration::from_secs(30), || async {
            std::fs::read_to_string(&log_path).unwrap_or_default().contains(&format!("directory updated epoch={epoch_with_all}")).then_some(())
        })
        .await;
    }

    // B2: /route vrací 3 nody
    let ek = SigningKey::generate(&mut OsRng);
    let ek_pub = ek.verifying_key().to_bytes();
    let route = eventually("/route", Duration::from_secs(60), || async { fetch_route(&http, &base, &ek_pub).await.ok() }).await;
    assert_eq!(route.nodes.len(), 3);
    let mut seen: Vec<_> = route.nodes.iter().map(|n| n.node_id.clone()).collect();
    seen.sort();
    seen.dedup();
    assert_eq!(seen.len(), 3, "route must return three distinct nodes");

    let by_host: HashMap<String, u16> = nodes.iter().map(|n| (n.hostname.clone(), n.port)).collect();
    let url_for = |host: &str| format!("ws://127.0.0.1:{}/client", by_host[host]);

    // klient se připojí s ticketem
    let key_doc: Value = http.get(format!("{base}/.well-known/ancord-key")).send().await.unwrap().json().await.unwrap();
    let worker_key: [u8; 32] = b64u_decode_array(key_doc["pubkey"].as_str().unwrap()).unwrap();
    let (first, mut ws) = connect_any(&route, &ek, &worker_key, &url_for).await.expect("admission with a valid ticket");
    ws.send_json(&json!({"t": "ping"})).await.unwrap();
    assert_eq!(ws.recv_json().await.unwrap()["t"], "pong");

    // stejný ticket podruhé: jednorázový
    let err = admit_unchecked(&url_for(&first.hostname), &first, &ek).await.expect_err("reused ticket must fail");
    assert!(err.to_string().contains("already used"), "{err}");

    // ticket jiného nodu
    let fresh = fetch_route(&http, &base, &ek_pub).await.unwrap();
    let (a, b) = (&fresh.nodes[0], &fresh.nodes[1]);
    let swapped = RouteNode { node_id: b.node_id.clone(), hostname: b.hostname.clone(), ticket: a.ticket.clone() };
    let err = admit_unchecked(&url_for(&b.hostname), &swapped, &ek).await.expect_err("ticket for another node must fail");
    assert!(err.to_string().contains("another node"), "{err}");
    // klient to pozná sám, ještě než se připojí
    let err = admit(&url_for(&b.hostname), &swapped, &ek, &worker_key).await.expect_err("client must refuse a foreign ticket");
    assert!(err.to_string().contains("untrusted route"), "{err}");
    // ticket podepsaný jiným klíčem než připnutým klíčem Workeru (podvržená trasa)
    let err = admit(&url_for(&a.hostname), a, &ek, &[7; 32]).await.expect_err("client must refuse a ticket not signed by the pinned worker key");
    assert!(err.to_string().contains("untrusted route"), "{err}");
    // route tvrdí, že na hostname sedí jiný node: ten svou identitu nedoloží
    let impostor = RouteNode { node_id: b.node_id.clone(), hostname: a.hostname.clone(), ticket: b.ticket.clone() };
    let err = admit(&url_for(&a.hostname), &impostor, &ek, &worker_key).await.expect_err("wrong node behind the hostname must fail");
    assert!(err.to_string().contains("unexpected hello"), "{err}");

    // špatný klíč ek: ticket bez soukromého klíče nestačí a nesmí se kvůli tomu spotřebovat
    let thief = SigningKey::generate(&mut OsRng);
    let err = admit_unchecked(&url_for(&a.hostname), a, &thief).await.expect_err("wrong ek must fail");
    assert!(err.to_string().contains("invalid client signature"), "{err}");
    admit(&url_for(&a.hostname), a, &ek, &worker_key).await.expect("ticket survives a failed attempt by someone without ek");

    // nesmysl místo ticketu
    let junk = RouteNode { ticket: "not-a-ticket".into(), ..b.clone() };
    let err = admit_unchecked(&url_for(&b.hostname), &junk, &ek).await.expect_err("junk ticket must fail");
    assert!(err.to_string().contains("malformed"), "{err}");

    // bez auth do limitu: 4408
    let (mut raw, _) = tokio_tungstenite::connect_async(url_for(&a.hostname)).await.unwrap();
    let hello = raw.next().await.unwrap().unwrap();
    assert!(hello.to_text().unwrap().contains("hello"));
    match tokio::time::timeout(Duration::from_secs(10), raw.next()).await.unwrap() {
        Some(Ok(Message::Close(Some(f)))) => assert_eq!(u16::from(f.code), 4408),
        other => panic!("expected close 4408, got {other:?}"),
    }

    // B3: Alice a Bob si píšou přes různé nody; node vidí jen ciphertext
    let secret = [0x5a; 32];
    let mut alice = Session::new(secret, Role::Initiator).unwrap();
    let mut bob = Session::new(secret, Role::Responder).unwrap();
    let (alice_ek, bob_ek) = (SigningKey::generate(&mut OsRng), SigningKey::generate(&mut OsRng));
    let alice_route = fetch_route(&http, &base, &alice_ek.verifying_key().to_bytes()).await.unwrap();
    let bob_route = fetch_route(&http, &base, &bob_ek.verifying_key().to_bytes()).await.unwrap();
    // různé vstupní nody: Alice první z trasy, Bob poslední
    let first_of = |r: &Route| r.nodes[0].node_id.clone();
    let (_, mut alice_ws) = connect_any(&alice_route, &alice_ek, &worker_key, &url_for).await.unwrap();
    let bob_reverse = Route { nodes: bob_route.nodes.iter().rev().cloned().collect(), ..bob_route.clone() };
    let (bob_node, mut bob_ws) = connect_any(&bob_reverse, &bob_ek, &worker_key, &url_for).await.unwrap();
    assert_ne!(first_of(&alice_route), bob_node.node_id, "test wants two different gateways");

    let hello = alice.prepare(b"ahoj Bobe").unwrap();
    assert!(transmit(&mut alice_ws, &hello).await.unwrap() >= 2);
    assert!(transmit(&mut alice_ws, &hello).await.unwrap() >= 2, "re-sending the same message is idempotent");
    let second = alice.prepare(b"druha zprava").unwrap();
    transmit(&mut alice_ws, &second).await.unwrap();
    let got = receive(&mut bob_ws, &mut bob).await.unwrap();
    assert_eq!(got.messages, vec![b"ahoj Bobe".to_vec(), b"druha zprava".to_vec()]);
    assert_eq!(got.rejected, 0);
    assert!(receive(&mut bob_ws, &mut bob).await.unwrap().messages.is_empty(), "delivered messages must not come back");

    let reply = bob.prepare(b"ahoj Alice").unwrap();
    transmit(&mut bob_ws, &reply).await.unwrap();
    assert_eq!(receive(&mut alice_ws, &mut alice).await.unwrap().messages, vec![b"ahoj Alice".to_vec()]);

    // B4: zpráva s push_id příjemce spustí wake od anchorů; Worker (dev) push zaloguje místo odeslání
    let push_key = SigningKey::generate(&mut OsRng);
    let push_id = base32_encode(blake3::hash(&push_key.verifying_key().to_bytes()).as_bytes());
    let reg_ts = unix_now();
    let payload = canonical_bytes(&json!({"op": "register", "platform": "fcm", "push_id": push_id, "token": "test-token", "ts": reg_ts})).unwrap();
    let reg = json!({
        "push_pubkey": b64u_encode(&push_key.verifying_key().to_bytes()),
        "platform": "fcm",
        "token": "test-token",
        "ts": reg_ts,
        "sig": b64u_encode(&sign(&push_key, "ancord/v1/push-reg", &payload)),
    });
    let res = http.post(format!("{base}/push/register")).json(&reg).send().await.unwrap();
    assert_eq!(res.status(), 200, "{}", res.text().await.unwrap_or_default());
    let wake_msg = alice.prepare(b"vzbud se").unwrap();
    transmit_with_push(&mut alice_ws, &wake_msg, Some(&push_id)).await.unwrap();
    eventually("wake reaches the worker and is queued as push", Duration::from_secs(60), || async {
        std::fs::read_to_string(logs.join("wrangler.log")).unwrap_or_default().contains("push_logged").then_some(())
    })
    .await;
    assert_eq!(receive(&mut bob_ws, &mut bob).await.unwrap().messages, vec![b"vzbud se".to_vec()]);

    // Bob nemůže číst z mailboxu, který patří jemu samému jako odesílateli
    let mut eve = Session::new([0x11; 32], Role::Responder).unwrap();
    assert!(receive(&mut bob_ws, &mut eve).await.unwrap().messages.is_empty());

    // B5: Bob je online (odběr u své gateway), Alice píše přes jiný node: anchor zprávu sám přepošle Bobovi
    subscribe(&mut bob_ws, &bob).await.unwrap();
    let live = alice.prepare(b"online doruceni").unwrap();
    transmit(&mut alice_ws, &live).await.unwrap();
    let pushed = next_pushed(&mut bob_ws, &mut bob, Duration::from_secs(10)).await.unwrap();
    assert_eq!(pushed, Some(b"online doruceni".to_vec()), "anchor must push the message to the recipient's gateway");
    assert!(receive(&mut bob_ws, &mut bob).await.unwrap().messages.is_empty(), "pushed and acked message must not come back");
    rpc(&mut bob_ws, json!({"t": "unsubscribe", "mailbox_id": b64u_encode(&bob.inbox())})).await.unwrap();

    // B6: uživatelé s veřejnými klíči: profil, vyhledání podle adresy, dohoda tajemství, typing, stav doručení, historie
    let (ua, ub) = (UserKeys::generate(unix_now()), UserKeys::generate(unix_now()));
    assert!(publish_profile(&mut alice_ws, &ua, 10).await.unwrap() >= 2);
    assert!(publish_profile(&mut bob_ws, &ub, 10).await.unwrap() >= 2);
    // Alice zahájí konverzaci jen podle Bobovy adresy, Bob obálku vyzvedne ze své schránky
    let mut sa = start_conversation(&mut alice_ws, &ua, &ub.pk()).await.unwrap();
    let mut ub = ub;
    let batch = process_inbox(&mut bob_ws, &mut ub).await.unwrap();
    assert_eq!((batch.accepted.len(), batch.postponed), (1, 0));
    ack_inbox(&mut bob_ws, &ub, &batch.done).await.unwrap();
    let mut sb = batch.accepted.into_iter().next().unwrap().session;
    assert_eq!(sa.secret, sb.secret);
    assert_ne!(sa.role, sb.role);
    assert!(process_inbox(&mut bob_ws, &mut ub).await.unwrap().accepted.is_empty(), "acked envelopes must not come back");
    let unknown = lookup_profile(&mut alice_ws, &UserKeys::generate(unix_now()).pk()).await;
    assert!(matches!(unknown, Err(CliError::Node { ref code, .. }) if code == "not_found"), "{unknown:?}");
    let mut lazy = ua.profile(unix_now(), 0).to_json();
    lazy["t"] = json!("profile_put");
    let bad_pow = rpc(&mut alice_ws, lazy).await;
    assert!(matches!(bad_pow, Err(CliError::Node { ref code, .. }) if code == "invalid_proof"), "{bad_pow:?}");

    subscribe_many(&mut bob_ws, &[&sb]).await.unwrap();
    send_typing(&mut alice_ws, &sa, true).await.unwrap();
    let event = next_event(&mut bob_ws, &mut sb, Duration::from_secs(10)).await.unwrap();
    assert_eq!(event, Some(Event::Typing(true)));
    let hello = sa.prepare(b"ahoj Bobe").unwrap();
    transmit(&mut alice_ws, &hello).await.unwrap();
    let event = next_event(&mut bob_ws, &mut sb, Duration::from_secs(10)).await.unwrap();
    assert_eq!(event, Some(Event::Message(b"ahoj Bobe".to_vec())));
    let until = Instant::now() + Duration::from_secs(10);
    let states = loop {
        let s = message_status(&mut alice_ws, &sa, &[hello.id]).await.unwrap();
        if s == ["delivered"] || Instant::now() > until {
            break s;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    };
    assert_eq!(states, ["delivered"]);
    assert_eq!((sa.history.len(), sa.history[0].outgoing), (1, true));
    assert_eq!((sb.history.len(), sb.history[0].outgoing), (1, false));
    rpc(&mut bob_ws, json!({"t": "unsubscribe", "mailbox_ids": [b64u_encode(&sb.inbox())]})).await.unwrap();

    // B8: celý tok podle adresy. Dave je offline s push tokenem; Carol ho najde podle adresy z QR kódu nebo textu.
    let pushes = || std::fs::read_to_string(logs.join("wrangler.log")).unwrap_or_default().matches("push_logged").count();
    let (carol, dave) = (UserKeys::generate(unix_now()), UserKeys::generate(unix_now()));
    let dave_ek = SigningKey::generate(&mut OsRng);
    let route = fetch_route(&http, &base, &dave_ek.verifying_key().to_bytes()).await.unwrap();
    let (_, mut dave_ws) = connect_any(&route, &dave_ek, &worker_key, &url_for).await.unwrap();
    assert!(publish_profile(&mut dave_ws, &dave, 10).await.unwrap() >= 2);
    let _ = dave_ws.close().await;
    register_push(&http, &base, dave.push_key(), "fcm", "dave-device", unix_now()).await.unwrap();
    assert!(publish_profile(&mut alice_ws, &carol, 10).await.unwrap() >= 2);
    let shared = address(&dave);
    assert!(shared.starts_with("ancord:"));
    let before = pushes();
    let mut sc = start_conversation(&mut alice_ws, &carol, &parse_address(&shared).unwrap()).await.unwrap();
    eventually("first contact wakes the offline recipient", Duration::from_secs(60), || async { (pushes() > before).then_some(()) }).await;
    transmit(&mut alice_ws, &sc.prepare(b"ahoj Dave").unwrap()).await.unwrap();

    // Dave se připojí: obálku přijme sám, odebírá novou konverzaci a vyzvedne, co čekalo
    let route = fetch_route(&http, &base, &dave_ek.verifying_key().to_bytes()).await.unwrap();
    let (_, mut dave_ws) = connect_any(&route, &dave_ek, &worker_key, &url_for).await.unwrap();
    let mut online = Online::new(dave, Vec::new(), 10);
    let (events, skipped) = online.attach(&mut dave_ws, &mut NoDisk).await.unwrap();
    assert!(skipped.is_empty());
    assert_eq!(events, vec![OnlineEvent::Contact { index: 0, watched: true }, OnlineEvent::Message { index: 0, text: b"ahoj Dave".to_vec() }]);
    assert_eq!(online.conversations[0].peer.as_deref(), Some(b64u_encode(&carol.pk()).as_str()));
    register_push(&http, &base, &online.conversations[0].inbox_push_key(), "fcm", "dave-device", unix_now()).await.unwrap();
    // online: zprávu anchor přepošle přes gateway, odpověď jde opačným směrem
    transmit(&mut alice_ws, &sc.prepare(b"jsi online?").unwrap()).await.unwrap();
    assert_eq!(next_online_message(&mut online, &mut dave_ws).await, b"jsi online?");
    online.send(&mut dave_ws, &mut NoDisk, 0, b"ano").await.unwrap();
    assert_eq!(receive(&mut alice_ws, &mut sc).await.unwrap().messages, vec![b"ano".to_vec()]);

    // Dave se odpojí: presence zanikne a další zpráva ho vzbudí pushem; po připojení si ji vyzvedne
    let _ = dave_ws.close().await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    let before = pushes();
    transmit(&mut alice_ws, &sc.prepare(b"jsi tam?").unwrap()).await.unwrap();
    eventually("message for an offline user wakes them", Duration::from_secs(60), || async { (pushes() > before).then_some(()) }).await;
    let route = fetch_route(&http, &base, &dave_ek.verifying_key().to_bytes()).await.unwrap();
    let (_, mut dave_ws) = connect_any(&route, &dave_ek, &worker_key, &url_for).await.unwrap();
    let (events, _) = online.attach(&mut dave_ws, &mut NoDisk).await.unwrap();
    assert_eq!(events, vec![OnlineEvent::Message { index: 0, text: b"jsi tam?".to_vec() }]);
    let _ = dave_ws.close().await;

    // zpráva, která přežije restart nodu (embedded úložiště) a čeká na Boba
    let pending = alice.prepare(b"pocka na restart").unwrap();
    transmit(&mut alice_ws, &pending).await.unwrap();
    let _ = (alice_ws.close().await, bob_ws.close().await);

    // restart zachová node_id a node zůstane aktivní
    let before = nodes[0].id.clone();
    nodes[0].proc = None;
    tokio::time::sleep(Duration::from_secs(1)).await;
    spawn_node(&mut nodes[0], worker_port, fwd_port, &logs, 1);
    let reloaded = Identity::load_or_create(&nodes[0].dir).unwrap();
    assert_eq!(reloaded.node_id(), before);
    // Worker ještě chvíli vidí starý heartbeat, proto se čeká na log nové instance
    let log_path = logs.join(format!("{}-1.log", &before[..8]));
    let log = eventually("restarted node reconnects", Duration::from_secs(60), || async {
        let log = std::fs::read_to_string(&log_path).unwrap_or_default();
        log.contains("control connection established").then_some(log)
    })
    .await;
    assert!(log.contains("status=active"), "restart must reuse the registration:\n{log}");
    // adresář přijde až po prvním heartbeatu; bez něj node data plane odmítá (`no_directory`)
    eventually("restarted node gets a directory", Duration::from_secs(60), || async {
        std::fs::read_to_string(&log_path).unwrap_or_default().contains("directory updated").then_some(())
    })
    .await;
    let list = admin_nodes(&http, &base).await;
    assert_eq!(list.len(), 3, "restart must not create a new node");
    assert!(list.iter().any(|n| n["node_id"] == before.as_str() && n["status"] == "active"));

    // B3: po restartu nodu zpráva stále čeká; Bob ji vyzvedne z libovolného nodu
    let mut bob_ws = eventually("fetch after restart", Duration::from_secs(60), || async {
        let route = fetch_route(&http, &base, &bob_ek.verifying_key().to_bytes()).await.ok()?;
        connect_any(&route, &bob_ek, &worker_key, &url_for).await.ok().map(|(_, ws)| ws)
    })
    .await;
    let end = Instant::now() + Duration::from_secs(60);
    let got = loop {
        // node po restartu ještě nemusí mít adresář, pak čtení selže s no_directory
        if let Ok(r) = receive(&mut bob_ws, &mut bob).await {
            if !r.messages.is_empty() {
                break r;
            }
        }
        assert!(Instant::now() < end, "timeout waiting for the message after restart");
        tokio::time::sleep(Duration::from_millis(500)).await;
    };
    assert_eq!(got.messages, vec![b"pocka na restart".to_vec()]);
    let _ = bob_ws.close().await;

    // jeden node vypnut: zápis i čtení dál fungují (kvorum 2 ze 3)
    nodes[1].proc = None;
    let alive = |route: Route, dead: &[&str]| Route { nodes: route.nodes.into_iter().filter(|n| !dead.contains(&n.node_id.as_str())).collect(), ..route };
    let dead1 = nodes[1].id.clone();
    let route = fetch_route(&http, &base, &alice_ek.verifying_key().to_bytes()).await.unwrap();
    let (_, mut ws) = connect_any(&alive(route, &[&dead1]), &alice_ek, &worker_key, &url_for).await.unwrap();
    let msg = alice.prepare(b"s jednim nodem dole").unwrap();
    let acks = transmit(&mut ws, &msg).await.expect("write with one anchor down");
    assert!(acks >= 2);
    let _ = ws.close().await;
    let route = fetch_route(&http, &base, &bob_ek.verifying_key().to_bytes()).await.unwrap();
    let (_, mut bob_ws) = connect_any(&alive(route, &[&dead1]), &bob_ek, &worker_key, &url_for).await.unwrap();
    assert_eq!(receive(&mut bob_ws, &mut bob).await.unwrap().messages, vec![b"s jednim nodem dole".to_vec()]);

    // druhý node vypnut: zbývá jeden anchor a zápis se odmítne
    nodes[2].proc = None;
    let dead2 = nodes[2].id.clone();
    let route = fetch_route(&http, &base, &alice_ek.verifying_key().to_bytes()).await.unwrap();
    let (_, mut ws) = connect_any(&alive(route, &[&dead1, &dead2]), &alice_ek, &worker_key, &url_for).await.unwrap();
    let refused = alice.prepare(b"nemelo by projit").unwrap();
    match transmit(&mut ws, &refused).await {
        Err(CliError::Node { code, .. }) => assert_eq!(code, "quorum_unavailable"),
        other => panic!("expected quorum_unavailable, got {other:?}"),
    }
    let _ = ws.close().await;
    fwd.abort();
}
