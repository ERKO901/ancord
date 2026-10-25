//! Data plane bez Workeru (`spec/PROTOCOL.md` §13): několik nodů v jednom procesu, adresář se nastavuje přímo.
//! Ověřuje kvorum, výpadky anchorů, idempotenci, ack, autorizaci node↔node a handoff po změně epochy.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;
use std::time::Duration;

use ancord_node::channel;
use ancord_node::config::Config;
use ancord_node::data::{self, DataError};
use ancord_node::encoding::b64u_encode;
use ancord_node::identity::Identity;
use ancord_node::maintenance;
use ancord_node::n2n;
use ancord_node::presence;
use ancord_node::server;
use ancord_node::sign::{auth_header, verify_request};
use ancord_node::state::NodeState;
use ancord_node::ticket::{DirectoryBody, DirectoryNode, DirectoryParams};
use ancord_node::time::unix_now;
use clap::Parser;
use serde_json::{json, Value};
use tokio::task::JoinHandle;

struct TestNode {
    state: Arc<NodeState>,
    host: String,
    server: JoinHandle<()>,
    _dir: tempfile::TempDir,
}

async fn spawn_node() -> TestNode {
    spawn_node_with_worker("http://127.0.0.1:1").await
}

async fn spawn_node_with_worker(worker_url: &str) -> TestNode {
    let dir = tempfile::tempdir().unwrap();
    let identity = Identity::load_or_create(dir.path()).unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let host = listener.local_addr().unwrap().to_string();
    let data_dir = dir.path().to_string_lossy().into_owned();
    let cfg = Config::parse_from([
        "ancord-node",
        "--worker-url",
        worker_url,
        "--data-dir",
        &data_dir,
        "--peer-url-template",
        "http://{host}",
        "--peer-timeout-s",
        "2",
    ]);
    let state = NodeState::new(cfg, identity, [0; 32]).unwrap();
    let app = server::router(state.clone());
    let server = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    TestNode { state, host, server, _dir: dir }
}

fn directory(epoch: u64, nodes: &[&TestNode]) -> DirectoryBody {
    let mut list: Vec<DirectoryNode> = nodes
        .iter()
        .map(|n| DirectoryNode { id: n.state.identity.node_id().to_owned(), pk: n.state.identity.pubkey_b64(), host: n.host.clone() })
        .collect();
    list.sort_by(|a, b| a.id.cmp(&b.id));
    DirectoryBody { v: 1, epoch, epoch_seconds: 300, issued_at: epoch * 300, params: DirectoryParams { r: 3, quorum: 2 }, nodes: list }
}

fn publish(dir: &DirectoryBody, to: &[&TestNode]) {
    for n in to {
        n.state.set_directory(dir.clone());
    }
}

const MB: [u8; 32] = [42; 32];

fn anchors_of(dir: &DirectoryBody, mailbox: &[u8; 32]) -> Vec<String> {
    ancord_node::hrw::anchors(mailbox, dir.nodes.iter().map(|n| n.id.as_str()), dir.epoch, 3)
}

async fn cluster(n: usize) -> Vec<TestNode> {
    cluster_with_worker(n, "http://127.0.0.1:1").await
}

async fn cluster_with_worker(n: usize, worker_url: &str) -> Vec<TestNode> {
    let mut nodes = Vec::new();
    for _ in 0..n {
        nodes.push(spawn_node_with_worker(worker_url).await);
    }
    let refs: Vec<&TestNode> = nodes.iter().collect();
    publish(&directory(1, &refs), &refs);
    nodes
}

fn stored_ids(list: &[data::Stored]) -> Vec<[u8; 16]> {
    list.iter().map(|m| m.id).collect()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn message_sent_via_one_node_is_fetched_via_another() {
    let nodes = cluster(3).await;
    let ts = unix_now();
    let acks = data::send(&nodes[0].state, MB, [1; 16], ts, b"ciphertext".to_vec()).await.unwrap();
    assert!(acks >= 2);
    let got = data::fetch(&nodes[2].state, MB, 100).await.unwrap();
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].ct, b"ciphertext");

    // opakovaný zápis téhož id nic nemění
    data::send(&nodes[1].state, MB, [1; 16], ts, b"different".to_vec()).await.unwrap();
    let got = data::fetch(&nodes[1].state, MB, 100).await.unwrap();
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].ct, b"ciphertext");

    // ack přes jiný node zprávu odstraní všude
    data::ack(&nodes[1].state, MB, vec![[1; 16]]).await.unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;
    for n in &nodes {
        assert!(data::fetch(&n.state, MB, 100).await.unwrap().is_empty());
    }
    // pozdní zápis zprávy, která už byla doručena, ji neoživí
    data::send(&nodes[0].state, MB, [1; 16], ts, b"ciphertext".to_vec()).await.unwrap();
    assert!(data::fetch(&nodes[0].state, MB, 100).await.unwrap().is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn one_anchor_down_still_works_two_down_rejects_writes() {
    let nodes = cluster(3).await;
    let dir = nodes[0].state.directory().unwrap();
    let anchors = anchors_of(&dir, &MB);
    assert_eq!(anchors.len(), 3);
    let ts = unix_now();
    data::send(&nodes[0].state, MB, [1; 16], ts, b"one".to_vec()).await.unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;

    // vypneme jeden anchor (ne ten, přes který půjdeme)
    nodes[1].server.abort();
    tokio::time::sleep(Duration::from_millis(100)).await;
    let via = &nodes[0].state;
    assert!(data::send(via, MB, [2; 16], ts, b"two".to_vec()).await.is_ok(), "write with 2 of 3 anchors must succeed");
    let got = data::fetch(via, MB, 100).await.unwrap();
    assert_eq!(stored_ids(&got), vec![[1; 16], [2; 16]], "read must still see both messages");

    // vypneme i druhý: zbývá jediný anchor, zápis i čtení se odmítnou
    nodes[2].server.abort();
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(data::send(via, MB, [3; 16], ts, b"three".to_vec()).await, Err(DataError::QuorumUnavailable));
    assert_eq!(data::fetch(via, MB, 100).await, Err(DataError::QuorumUnavailable));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn rejects_when_directory_is_missing_or_too_small() {
    let lonely = spawn_node().await;
    assert_eq!(data::send(&lonely.state, MB, [1; 16], unix_now(), vec![1]).await, Err(DataError::NoDirectory));
    publish(&directory(1, &[&lonely]), &[&lonely]);
    assert_eq!(data::send(&lonely.state, MB, [1; 16], unix_now(), vec![1]).await, Err(DataError::InsufficientNodes));
}

fn dnode(n: &TestNode) -> DirectoryNode {
    DirectoryNode { id: n.state.identity.node_id().to_owned(), pk: n.state.identity.pubkey_b64(), host: n.host.clone() }
}

/// Operace na jiném nodu přes zabezpečený kanál (`PROTOCOL.md` §14.2): `(stav, tělo)`.
async fn call_peer(from: &TestNode, to: &TestNode, op: &str, body: &Value) -> (u16, Value) {
    n2n::call(&from.state, &dnode(to), op, body).await.unwrap()
}

fn put_body(mailbox: &[u8; 32]) -> Value {
    json!({"mailbox_id": b64u_encode(mailbox), "id": b64u_encode(&[9; 16]), "ts": unix_now(), "exp": unix_now() + 1000, "ct": b64u_encode(b"x")})
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn peer_api_checks_membership_and_anchor_role() {
    let nodes = cluster(3).await;
    let outsider = spawn_node().await;
    let dir = nodes[0].state.directory().unwrap();
    let anchor_ids = anchors_of(&dir, &MB);
    let target = nodes.iter().find(|n| anchor_ids.iter().any(|a| a == n.state.identity.node_id())).unwrap();

    // člen adresáře, který je anchorem: projde
    let (status, body) = call_peer(&nodes[0], target, "put", &put_body(&MB)).await;
    assert_eq!((status, body["status"].as_str()), (200, Some("stored")));
    let (_, again) = call_peer(&nodes[0], target, "put", &put_body(&MB)).await;
    assert_eq!(again["status"], "duplicate");

    // cizí node mimo adresář neprojde ani handshakem
    assert_eq!(call_peer(&outsider, target, "put", &put_body(&MB)).await.0, 403);
    assert_eq!(call_peer(&outsider, target, "get", &json!({"mailbox_id": b64u_encode(&MB)})).await.0, 403);

    // neznámá operace a nesmysl uvnitř kanálu
    assert_eq!(call_peer(&nodes[0], target, "drop_everything", &json!({})).await.0, 400);
    assert_eq!(call_peer(&nodes[0], target, "get", &json!({"mailbox_id": 5})).await.0, 400);
}

/// Ruční iniciátor, který nepoužívá `n2n.rs`: ověřuje responder nezávisle na naší implementaci volajícího.
struct ManualSession {
    sid: [u8; 16],
    keys: channel::SessionKeys,
}

async fn manual_handshake(from: &TestNode, to: &TestNode) -> ManualSession {
    let ce = channel::Ephemeral::generate();
    let ce_pub = ce.public();
    let bytes = serde_json::to_vec(&json!({"ce": b64u_encode(&ce_pub)})).unwrap();
    let nonce: [u8; 16] = rand::random();
    let header = auth_header(from.state.identity.signing_key(), "POST", "/n/hs", &bytes, unix_now(), &nonce);
    let res = reqwest::Client::new().post(format!("http://{}/n/hs", to.host)).header("authorization", header).body(bytes).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let a: Value = res.json().await.unwrap();
    let ne: [u8; 32] = ancord_node::encoding::b64u_decode_array(a["ne"].as_str().unwrap()).unwrap();
    let sid: [u8; 16] = ancord_node::encoding::b64u_decode_array(a["sid"].as_str().unwrap()).unwrap();
    let sig: [u8; 64] = ancord_node::encoding::b64u_decode_array(a["sig"].as_str().unwrap()).unwrap();
    let th = channel::n2n_transcript(&from.state.identity.pubkey(), &to.state.identity.pubkey(), &nonce, &ce_pub, &ne, &sid);
    assert!(ancord_node::sign::verify(&to.state.identity.pubkey(), ancord_node::sign::DOMAIN_N2N_RESP, &th, &sig));
    ManualSession { sid, keys: channel::SessionKeys::derive(&ce.diffie_hellman(&ne).unwrap(), &th) }
}

fn envelope(s: &ManualSession, nonce: &[u8; 24], request: &Value) -> Value {
    let c = channel::seal_request(&s.keys.i2r, &s.sid, nonce, request.to_string().as_bytes()).unwrap();
    json!({"sid": b64u_encode(&s.sid), "n": b64u_encode(nonce), "c": b64u_encode(&c)})
}

async fn post_x(to: &TestNode, body: &Value) -> reqwest::Response {
    reqwest::Client::new().post(format!("http://{}/n/x", to.host)).json(body).send().await.unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn peer_channel_rejects_plaintext_tamper_replay_and_unknown_session() {
    let nodes = cluster(3).await;
    let dir = nodes[0].state.directory().unwrap();
    let anchor_ids = anchors_of(&dir, &MB);
    let target = nodes.iter().find(|n| anchor_ids.iter().any(|a| a == n.state.identity.node_id())).unwrap();
    let http = reqwest::Client::new();

    // staré holé API už neexistuje, ani s platným podpisem
    for path in ["/n/put", "/n/get", "/n/ack", "/n/presence", "/n/deliver"] {
        let bytes = serde_json::to_vec(&put_body(&MB)).unwrap();
        let header = auth_header(nodes[0].state.identity.signing_key(), "POST", path, &bytes, unix_now(), &rand::random());
        let res = http.post(format!("http://{}{path}", target.host)).header("authorization", header).body(bytes).send().await.unwrap();
        assert!(res.status() == 404 || res.status() == 405, "{path} must not exist in plaintext, got {}", res.status());
    }

    // handshake: bez podpisu, s cizím klíčem mimo adresář a replay
    let hs_body = serde_json::to_vec(&json!({"ce": b64u_encode(&channel::Ephemeral::generate().public())})).unwrap();
    let bare = http.post(format!("http://{}/n/hs", target.host)).body(hs_body.clone()).send().await.unwrap();
    assert_eq!(bare.status(), 401);
    let header = auth_header(nodes[0].state.identity.signing_key(), "POST", "/n/hs", &hs_body, unix_now(), &[5; 16]);
    let hs = || http.post(format!("http://{}/n/hs", target.host)).header("authorization", header.clone()).body(hs_body.clone()).send();
    assert_eq!(hs().await.unwrap().status(), 200);
    assert_eq!(hs().await.unwrap().status(), 409, "replayed handshake");
    let low_order = serde_json::to_vec(&json!({"ce": b64u_encode(&[0u8; 32])})).unwrap();
    let header = auth_header(nodes[0].state.identity.signing_key(), "POST", "/n/hs", &low_order, unix_now(), &[6; 16]);
    let res = http.post(format!("http://{}/n/hs", target.host)).header("authorization", header).body(low_order).send().await.unwrap();
    assert_eq!(res.status(), 400, "low order point must be refused");

    // šifrované požadavky
    let s = manual_handshake(&nodes[0], target).await;
    let get = json!({"op": "get", "ts": unix_now(), "body": {"mailbox_id": b64u_encode(&MB)}});
    let nonce: [u8; 24] = rand::random();
    let env = envelope(&s, &nonce, &get);
    let ok = post_x(target, &env).await;
    assert_eq!(ok.status(), 200);
    let answer: Value = ok.json().await.unwrap();
    let rn: [u8; 24] = ancord_node::encoding::b64u_decode_array(answer["n"].as_str().unwrap()).unwrap();
    let rc = ancord_node::encoding::b64u_decode(answer["c"].as_str().unwrap()).unwrap();
    let plain = channel::open_response(&s.keys.r2i, &s.sid, &nonce, &rn, &rc).unwrap();
    assert_eq!(serde_json::from_slice::<Value>(&plain).unwrap()["status"], 200);

    // replay téhož obálky: odpověď nese 409 uvnitř kanálu
    let replay: Value = post_x(target, &env).await.json().await.unwrap();
    let rn: [u8; 24] = ancord_node::encoding::b64u_decode_array(replay["n"].as_str().unwrap()).unwrap();
    let rc = ancord_node::encoding::b64u_decode(replay["c"].as_str().unwrap()).unwrap();
    let plain = channel::open_response(&s.keys.r2i, &s.sid, &nonce, &rn, &rc).unwrap();
    assert_eq!(serde_json::from_slice::<Value>(&plain).unwrap()["status"], 409);

    // odpověď nejde přiřadit k jinému požadavku
    assert!(channel::open_response(&s.keys.r2i, &s.sid, &[0; 24], &rn, &rc).is_err());

    // porušený šifrový text, cizí session, starý čas
    let mut tampered = envelope(&s, &rand::random(), &get);
    let mut c = ancord_node::encoding::b64u_decode(tampered["c"].as_str().unwrap()).unwrap();
    c[0] ^= 1;
    tampered["c"] = json!(b64u_encode(&c));
    assert_eq!(post_x(target, &tampered).await.status(), 401);
    let mut foreign = envelope(&s, &rand::random(), &get);
    foreign["sid"] = json!(b64u_encode(&[7; 16]));
    let res = post_x(target, &foreign).await;
    assert_eq!(res.status(), 401);
    assert_eq!(res.json::<Value>().await.unwrap()["error"]["code"], "unknown_session");
    let stale = json!({"op": "get", "ts": unix_now() - 3600, "body": {"mailbox_id": b64u_encode(&MB)}});
    let n3: [u8; 24] = rand::random();
    let reply: Value = post_x(target, &envelope(&s, &n3, &stale)).await.json().await.unwrap();
    let rn: [u8; 24] = ancord_node::encoding::b64u_decode_array(reply["n"].as_str().unwrap()).unwrap();
    let rc = ancord_node::encoding::b64u_decode(reply["c"].as_str().unwrap()).unwrap();
    let plain = channel::open_response(&s.keys.r2i, &s.sid, &n3, &rn, &rc).unwrap();
    assert_eq!(serde_json::from_slice::<Value>(&plain).unwrap()["status"], 400, "stale timestamp");
    assert_eq!(post_x(target, &json!({"sid": "x"})).await.status(), 400);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn caller_rejects_responder_that_cannot_prove_identity() {
    let nodes = cluster(2).await;
    // „node“ na cestě (třeba Cloudflare), který odpoví vlastním podpisem místo podpisu identity z adresáře
    let impostor_key = ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng);
    let app = axum::Router::new().route(
        "/n/hs",
        axum::routing::post(move || {
            let key = impostor_key.clone();
            async move {
                let sig = ancord_node::sign::sign(&key, ancord_node::sign::DOMAIN_N2N_RESP, &[0; 32]);
                axum::Json(json!({"ne": b64u_encode(&channel::Ephemeral::generate().public()), "sid": b64u_encode(&[1; 16]), "sig": b64u_encode(&sig)}))
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mut victim = dnode(&nodes[1]);
    victim.host = listener.local_addr().unwrap().to_string();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    let err = n2n::call(&nodes[0].state, &victim, "get", &json!({})).await.unwrap_err();
    assert!(err.contains("prove its identity"), "{err}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn non_anchor_refuses_foreign_mailbox() {
    // se čtyřmi nody a r = 3 existuje mailbox, jehož anchorem node není
    let nodes = cluster(4).await;
    let dir = nodes[0].state.directory().unwrap();
    let (mailbox, outsider_id) = (0u8..=255)
        .map(|b| [b; 32])
        .find_map(|mb| {
            let a = anchors_of(&dir, &mb);
            dir.nodes.iter().find(|n| !a.contains(&n.id)).map(|n| (mb, n.id.clone()))
        })
        .unwrap();
    let non_anchor = nodes.iter().find(|n| n.state.identity.node_id() == outsider_id).unwrap();
    let caller = nodes.iter().find(|n| n.state.identity.node_id() != outsider_id).unwrap();
    assert_eq!(call_peer(caller, non_anchor, "put", &put_body(&mailbox)).await.0, 403);
    // přesto lze zprávu poslat přes tento node: koordinátor nemusí být anchor
    data::send(&non_anchor.state, mailbox, [1; 16], unix_now(), b"via".to_vec()).await.unwrap();
    assert_eq!(data::fetch(&non_anchor.state, mailbox, 10).await.unwrap().len(), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn handoff_moves_data_to_new_anchors_when_epoch_changes() {
    let nodes = cluster(4).await;
    let refs: Vec<&TestNode> = nodes.iter().collect();
    let dir1 = nodes[0].state.directory().unwrap();
    let old: Vec<String> = anchors_of(&dir1, &MB);

    // najdi epochu, ve které se sada anchorů změní
    let epoch2 = (2u64..200).find(|e| anchors_of(&directory(*e, &refs), &MB) != old).unwrap();
    let ts = unix_now();
    data::send(&nodes[0].state, MB, [1; 16], ts, b"keep me".to_vec()).await.unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
    // druhá zpráva dostane ack, takže se přenáší jako náhrobek
    data::send(&nodes[0].state, MB, [2; 16], ts, b"delivered".to_vec()).await.unwrap();
    data::ack(&nodes[0].state, MB, vec![[2; 16]]).await.unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;

    let dir2 = directory(epoch2, &refs);
    let new = anchors_of(&dir2, &MB);
    publish(&dir2, &refs);
    for n in &nodes {
        assert!(maintenance::handoff_if_needed(&n.state).await);
    }
    for n in &nodes {
        let id = n.state.identity.node_id();
        let records = n.state.store.records(&MB).unwrap();
        if new.iter().any(|a| a == id) {
            let live: Vec<_> = records.iter().filter(|r| r.ct.is_some()).collect();
            assert_eq!(live.len(), 1, "new anchor {id} must hold the pending message");
            assert!(records.iter().any(|r| r.id == [2; 16] && r.ct.is_none()), "tombstone must be handed off too");
        } else {
            assert!(records.is_empty(), "former anchor {id} must give the data up");
        }
    }
    // po handoffu je zpráva čitelná a doručená se neobjeví
    let got = data::fetch(&nodes[3].state, MB, 10).await.unwrap();
    assert_eq!(stored_ids(&got), vec![[1; 16]]);
    // druhý průchod už nic nedělá
    assert!(!maintenance::handoff_if_needed(&nodes[0].state).await);
}

type WakeLog = Arc<std::sync::Mutex<Vec<([u8; 32], Value)>>>;

/// Falešný Worker: zapisuje ověřené `/wake` requesty (pubkey volajícího, tělo).
struct MockWorker {
    url: String,
    wakes: WakeLog,
}

async fn mock_worker() -> MockWorker {
    use axum::body::Bytes;
    use axum::http::{HeaderMap, StatusCode, Uri};
    use axum::routing::post;

    let wakes: WakeLog = Arc::default();
    let app = axum::Router::new()
        .route(
            "/wake",
            post(|axum::extract::State(log): axum::extract::State<WakeLog>, uri: Uri, headers: HeaderMap, body: Bytes| async move {
                let path = uri.path_and_query().map_or("/", |p| p.as_str());
                let auth = headers.get("authorization").and_then(|v| v.to_str().ok()).and_then(|h| verify_request(h, "POST", path, &body, unix_now(), 60));
                let Some(auth) = auth else { return (StatusCode::UNAUTHORIZED, "{}".to_owned()) };
                log.lock().unwrap().push((auth.pubkey, serde_json::from_slice(&body).unwrap()));
                (StatusCode::ACCEPTED, json!({"status": "queued"}).to_string())
            }),
        )
        .with_state(wakes.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    MockWorker { url, wakes }
}

async fn wait_for_wakes(mock: &MockWorker, at_least: usize) -> Vec<([u8; 32], Value)> {
    for _ in 0..50 {
        let got = mock.wakes.lock().unwrap().clone();
        if got.len() >= at_least {
            return got;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    mock.wakes.lock().unwrap().clone()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn anchors_send_signed_wake_only_for_new_messages_with_push_id() {
    let mock = mock_worker().await;
    let nodes = cluster_with_worker(3, &mock.url).await;
    let push_id = "a".repeat(52);
    let ts = unix_now();

    // bez push_id žádný wake
    data::send(&nodes[0].state, MB, [1; 16], ts, b"quiet".to_vec()).await.unwrap();
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert!(mock.wakes.lock().unwrap().is_empty());

    // s push_id pošle wake každý ze 3 anchorů, podepsaný vlastní identitou
    data::send_with_push(&nodes[1].state, MB, [2; 16], ts, b"loud".to_vec(), Some(push_id.clone())).await.unwrap();
    let wakes = wait_for_wakes(&mock, 3).await;
    assert_eq!(wakes.len(), 3, "each of the 3 anchors wakes once");
    let mut callers: Vec<[u8; 32]> = wakes.iter().map(|(pk, _)| *pk).collect();
    callers.sort();
    callers.dedup();
    assert_eq!(callers.len(), 3);
    for (pk, body) in &wakes {
        assert!(nodes.iter().any(|n| &n.state.identity.pubkey() == pk));
        assert_eq!(body["push_id"], push_id.as_str());
        assert_eq!(body["mailbox_id"], b64u_encode(&MB));
    }

    // duplicitní zápis wake nespouští (a lokální koalescence by ho stejně potlačila)
    data::send_with_push(&nodes[2].state, MB, [2; 16], ts, b"loud".to_vec(), Some(push_id)).await.unwrap();
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(mock.wakes.lock().unwrap().len(), 3);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn local_coalescing_limits_wakes_per_push_id() {
    let mock = mock_worker().await;
    let nodes = cluster_with_worker(3, &mock.url).await;
    let push_id = "b".repeat(52);
    for i in 1u8..=3 {
        data::send_with_push(&nodes[0].state, MB, [i; 16], unix_now(), vec![i], Some(push_id.clone())).await.unwrap();
    }
    wait_for_wakes(&mock, 3).await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(mock.wakes.lock().unwrap().len(), 3, "three messages, one wake per anchor within the interval");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn malformed_push_id_is_rejected_by_peer_api() {
    let nodes = cluster(3).await;
    let dir = nodes[0].state.directory().unwrap();
    let anchor_ids = anchors_of(&dir, &MB);
    let target = nodes.iter().find(|n| anchor_ids.iter().any(|a| a == n.state.identity.node_id())).unwrap();
    let mut body = put_body(&MB);
    body["push_id"] = json!("not-a-push-id");
    assert_eq!(call_peer(&nodes[0], target, "put", &body).await.0, 400);
}

fn subscribe(node: &TestNode, conn: u64) -> tokio::sync::mpsc::Receiver<Value> {
    let (tx, rx) = tokio::sync::mpsc::channel(16);
    assert!(presence::add_subscriber(&node.state, conn, tx, MB));
    rx
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn online_recipient_gets_message_pushed_and_no_wake() {
    let mock = mock_worker().await;
    let nodes = cluster_with_worker(4, &mock.url).await;
    let push_id = "c".repeat(52);
    // příjemce je připojen na nodes[0] (gateway), odesílatel píše přes nodes[3]
    let mut inbox = subscribe(&nodes[0], 1);
    assert!(presence::announce(&nodes[0].state, MB, true).await >= 2);

    data::send_with_push(&nodes[3].state, MB, [1; 16], unix_now(), b"hello".to_vec(), Some(push_id.clone())).await.unwrap();
    let pushed = tokio::time::timeout(Duration::from_secs(3), inbox.recv()).await.unwrap().unwrap();
    assert_eq!(pushed["t"], "message");
    assert_eq!(pushed["id"], b64u_encode(&[1; 16]));
    assert_eq!(pushed["ct"], b64u_encode(b"hello"));

    // od více anchorů přijde zpráva jednou
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert!(inbox.try_recv().is_err(), "duplicates from other anchors must be dropped");
    assert!(mock.wakes.lock().unwrap().is_empty(), "online delivery must not wake");
    // zpráva zůstává pending do klientského ack
    assert_eq!(data::fetch(&nodes[1].state, MB, 10).await.unwrap().len(), 1);

    // po zrušení odběru a presence se wake vrací
    assert!(presence::remove_subscriber(&nodes[0].state, 1, MB));
    presence::announce(&nodes[0].state, MB, false).await;
    data::send_with_push(&nodes[3].state, MB, [2; 16], unix_now(), b"later".to_vec(), Some(push_id)).await.unwrap();
    assert!(!wait_for_wakes(&mock, 1).await.is_empty(), "offline recipient must be woken");
    assert!(inbox.try_recv().is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn stale_presence_falls_back_to_wake() {
    let mock = mock_worker().await;
    let nodes = cluster_with_worker(3, &mock.url).await;
    // presence existuje, ale gateway už odběratele nemá (spojení zaniklo bez zrušení presence)
    let inbox = subscribe(&nodes[0], 1);
    presence::announce(&nodes[0].state, MB, true).await;
    drop(inbox);

    data::send_with_push(&nodes[1].state, MB, [1; 16], unix_now(), b"x".to_vec(), Some("d".repeat(52))).await.unwrap();
    assert!(!wait_for_wakes(&mock, 1).await.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn only_anchors_may_deliver_and_presence_needs_membership() {
    let nodes = cluster(4).await;
    let dir = nodes[0].state.directory().unwrap();
    let anchor_ids = anchors_of(&dir, &MB);
    let (anchors, others): (Vec<&TestNode>, Vec<&TestNode>) = nodes.iter().partition(|n| anchor_ids.iter().any(|a| a == n.state.identity.node_id()));
    let gateway = others[0];
    let _inbox = subscribe(gateway, 1);
    let body = json!({"mailbox_id": b64u_encode(&MB), "id": b64u_encode(&[9; 16]), "ts": unix_now(), "ct": b64u_encode(b"x")});

    // doručit do gateway smí jen anchor mailboxu
    assert_eq!(call_peer(anchors[0], gateway, "deliver", &body).await.0, 200);
    assert_eq!(call_peer(gateway, gateway, "deliver", &body).await.0, 403, "a non-anchor cannot push into a gateway");

    // presence smí oznámit jen člen adresáře, a jen anchoru
    let presence_body = json!({"mailbox_id": b64u_encode(&MB), "exp": unix_now() + 30});
    assert_eq!(call_peer(gateway, anchors[0], "presence", &presence_body).await.0, 200);
    assert_eq!(call_peer(gateway, gateway, "presence", &presence_body).await.0, 403, "not an anchor");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn session_is_reestablished_after_responder_forgets_it() {
    let nodes = cluster(3).await;
    let dir = nodes[0].state.directory().unwrap();
    let anchor_ids = anchors_of(&dir, &MB);
    let target = nodes.iter().find(|n| anchor_ids.iter().any(|a| a == n.state.identity.node_id())).unwrap();
    let get = json!({"mailbox_id": b64u_encode(&MB)});
    assert_eq!(call_peer(&nodes[0], target, "get", &get).await.0, 200);
    // restart responderu: session zmizí z paměti
    target.state.n2n_in.lock().unwrap().clear();
    assert_eq!(call_peer(&nodes[0], target, "get", &get).await.0, 200, "caller must renegotiate transparently");
}
