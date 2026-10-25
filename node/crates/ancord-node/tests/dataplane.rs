//! Data plane bez Workeru (`spec/PROTOCOL.md` §13): několik nodů v jednom procesu, adresář se nastavuje přímo.
//! Ověřuje kvorum, výpadky anchorů, idempotenci, ack, autorizaci node↔node a handoff po změně epochy.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;
use std::time::Duration;

use ancord_node::config::Config;
use ancord_node::data::{self, DataError};
use ancord_node::encoding::b64u_encode;
use ancord_node::identity::Identity;
use ancord_node::maintenance;
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

async fn signed_post(from: &TestNode, to: &TestNode, path: &str, body: &Value) -> reqwest::Response {
    let bytes = serde_json::to_vec(body).unwrap();
    let header = auth_header(from.state.identity.signing_key(), "POST", path, &bytes, unix_now(), &rand::random());
    reqwest::Client::new().post(format!("http://{}{path}", to.host)).header("authorization", header).body(bytes).send().await.unwrap()
}

fn put_body(mailbox: &[u8; 32]) -> Value {
    json!({"mailbox_id": b64u_encode(mailbox), "id": b64u_encode(&[9; 16]), "ts": unix_now(), "exp": unix_now() + 1000, "ct": b64u_encode(b"x")})
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn peer_api_checks_signature_membership_and_anchor_role() {
    let nodes = cluster(3).await;
    let outsider = spawn_node().await;
    let dir = nodes[0].state.directory().unwrap();
    let anchor_ids = anchors_of(&dir, &MB);
    let target = nodes.iter().find(|n| anchor_ids.iter().any(|a| a == n.state.identity.node_id())).unwrap();

    // člen adresáře, který je anchorem: projde
    let ok = signed_post(&nodes[0], target, "/n/put", &put_body(&MB)).await;
    assert_eq!(ok.status(), 200);
    assert_eq!(ok.json::<Value>().await.unwrap()["status"], "stored");
    let again = signed_post(&nodes[0], target, "/n/put", &put_body(&MB)).await;
    assert_eq!(again.json::<Value>().await.unwrap()["status"], "duplicate");

    // cizí node mimo adresář
    assert_eq!(signed_post(&outsider, target, "/n/put", &put_body(&MB)).await.status(), 403);
    assert_eq!(signed_post(&outsider, target, "/n/get", &json!({"mailbox_id": b64u_encode(&MB)})).await.status(), 403);

    // bez podpisu
    let bare = reqwest::Client::new().post(format!("http://{}/n/get", target.host)).body("{}").send().await.unwrap();
    assert_eq!(bare.status(), 401);

    // replay stejného requestu
    let bytes = serde_json::to_vec(&json!({"mailbox_id": b64u_encode(&MB)})).unwrap();
    let header = auth_header(nodes[0].state.identity.signing_key(), "POST", "/n/get", &bytes, unix_now(), &[5; 16]);
    let send = || reqwest::Client::new().post(format!("http://{}/n/get", target.host)).header("authorization", header.clone()).body(bytes.clone()).send();
    assert_eq!(send().await.unwrap().status(), 200);
    assert_eq!(send().await.unwrap().status(), 409);
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
    let res = signed_post(caller, non_anchor, "/n/put", &put_body(&mailbox)).await;
    assert_eq!(res.status(), 403);
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
    assert_eq!(signed_post(&nodes[0], target, "/n/put", &body).await.status(), 400);
}
