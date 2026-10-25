//! Kritéria přijetí B1 a B2 (`ARCHITECTURE.md` §12.4): tři nody proti lokálně spuštěnému Workeru (`wrangler dev`, mock Cloudflare).
//!
//! Test potřebuje nainstalované závislosti Workeru (`cd worker && npm install`); bez nich se přeskočí.
//! `ANCORD_SKIP_E2E=1` ho vypne.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};

use ancord_cli::{admit, connect_any, fetch_route, RouteNode};
use ancord_node::identity::{hostname_of, Identity};
use axum::extract::State;
use axum::http::{StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::Router;
use ed25519_dalek::SigningKey;
use futures_util::{SinkExt, StreamExt};
use rand::rngs::OsRng;
use serde_json::Value;
use tokio::process::{Child, Command};
use tokio_tungstenite::tungstenite::Message;

const ZONE: &str = "melonet.cz";
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

fn spawn_node(n: &mut NodeProc, worker_port: u16, log_dir: &Path, generation: u32) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ancord-node"));
    cmd.env("WORKER_URL", format!("http://127.0.0.1:{worker_port}"))
        .env("DATA_DIR", &n.dir)
        .env("LISTEN", format!("127.0.0.1:{}", n.port))
        .env("CLOUDFLARED_BIN", "none")
        .env("CLAIM_CPU_CORES", "2")
        .env("CLAIM_RAM_MB", "256")
        .env("CLAIM_STORAGE_GB", "1")
        .env("CLAIM_BANDWIDTH_MBPS", "100")
        .env("AUTH_TIMEOUT_S", "2")
        .env("RUST_LOG", "info");
    n.proc = Some(Proc::spawn(cmd, &log_dir.join(format!("{}-{generation}.log", &n.id[..8]))));
}

async fn forward(State(ports): State<Arc<HashMap<String, u16>>>, uri: Uri) -> Response {
    let pq = uri.path_and_query().map_or("", |p| p.as_str());
    let Some((host, tail)) = pq.trim_start_matches('/').split_once('/') else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let Some(port) = ports.get(host) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    match reqwest::get(format!("http://127.0.0.1:{port}/{tail}")).await {
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
    let overrides = r#"{"snapshotRefreshS":2,"challenges":{"cpu":{"iters":100000,"every":1000},"space":{"ram":{"maxLeaves":4096},"storage":{"maxLeaves":16384}}}}"#;
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
        spawn_node(n, worker_port, &logs, 0);
    }

    // B1: všechny tři nody aktivní a s naměřenou kapacitou
    eventually("3 active nodes with measured capacity", Duration::from_secs(240), || async {
        let list = admin_nodes(&http, &base).await;
        let ok = list.len() == 3
            && list.iter().all(|n| n["status"] == "active" && n["live"] == true && n["challenges_ok"].as_u64().unwrap_or(0) >= 4 && n["strikes"] == 0);
        ok.then_some(())
    })
    .await;

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
    let (first, mut ws) = connect_any(&route, &ek, &url_for).await.expect("admission with a valid ticket");
    ws.send(Message::text(r#"{"t":"ping"}"#)).await.unwrap();
    let pong = ws.next().await.unwrap().unwrap();
    assert!(pong.to_text().unwrap().contains("pong"));

    // stejný ticket podruhé: jednorázový
    let err = admit(&url_for(&first.hostname), &first, &ek).await.expect_err("reused ticket must fail");
    assert!(err.to_string().contains("already used"), "{err}");

    // ticket jiného nodu
    let fresh = fetch_route(&http, &base, &ek_pub).await.unwrap();
    let (a, b) = (&fresh.nodes[0], &fresh.nodes[1]);
    let swapped = RouteNode { node_id: b.node_id.clone(), hostname: b.hostname.clone(), ticket: a.ticket.clone() };
    let err = admit(&url_for(&b.hostname), &swapped, &ek).await.expect_err("ticket for another node must fail");
    assert!(err.to_string().contains("another node"), "{err}");

    // špatný klíč ek: ticket bez soukromého klíče nestačí a nesmí se kvůli tomu spotřebovat
    let thief = SigningKey::generate(&mut OsRng);
    let err = admit(&url_for(&a.hostname), a, &thief).await.expect_err("wrong ek must fail");
    assert!(err.to_string().contains("invalid client signature"), "{err}");
    admit(&url_for(&a.hostname), a, &ek).await.expect("ticket survives a failed attempt by someone without ek");

    // nesmysl místo ticketu
    let junk = RouteNode { ticket: "not-a-ticket".into(), ..b.clone() };
    let err = admit(&url_for(&b.hostname), &junk, &ek).await.expect_err("junk ticket must fail");
    assert!(err.to_string().contains("malformed"), "{err}");

    // bez auth do limitu: 4408
    let (mut raw, _) = tokio_tungstenite::connect_async(url_for(&a.hostname)).await.unwrap();
    let hello = raw.next().await.unwrap().unwrap();
    assert!(hello.to_text().unwrap().contains("hello"));
    match tokio::time::timeout(Duration::from_secs(10), raw.next()).await.unwrap() {
        Some(Ok(Message::Close(Some(f)))) => assert_eq!(u16::from(f.code), 4408),
        other => panic!("expected close 4408, got {other:?}"),
    }

    // restart zachová node_id a node zůstane aktivní
    let before = nodes[0].id.clone();
    nodes[0].proc = None;
    tokio::time::sleep(Duration::from_secs(1)).await;
    spawn_node(&mut nodes[0], worker_port, &logs, 1);
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
    let list = admin_nodes(&http, &base).await;
    assert_eq!(list.len(), 3, "restart must not create a new node");
    assert!(list.iter().any(|n| n["node_id"] == before.as_str() && n["status"] == "active"));
    fwd.abort();
}
