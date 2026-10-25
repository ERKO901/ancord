//! Minimální klient: `GET /route`, připojení na node s ticketem a failover (`spec/PROTOCOL.md` §8 a §8.1).

use std::time::Duration;

use ancord_node::encoding::{b64u_decode_array, b64u_encode};
use ancord_node::ratchet::{Direction, Receiver, Sender};
use ancord_node::sign::{sign, DOMAIN_CLIENT_AUTH};
use ed25519_dalek::SigningKey;
use futures_util::{SinkExt, StreamExt};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub type NodeSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("route: {0}")]
    Route(String),
    #[error("{node}: {reason}")]
    Admission { node: String, reason: String },
    #[error("no node accepted the connection: {0}")]
    AllFailed(String),
    #[error("node refused the request ({code}): {reason}")]
    Node { code: String, reason: String },
    #[error("connection: {0}")]
    Connection(String),
    #[error("session file: {0}")]
    Session(String),
}

#[derive(Debug, Clone, Deserialize)]
pub struct RouteNode {
    pub node_id: String,
    pub hostname: String,
    pub ticket: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Route {
    pub epoch: u64,
    pub ttl: u64,
    pub nodes: Vec<RouteNode>,
}

/// `GET /route?ek=…` (spec §8).
pub async fn fetch_route(http: &reqwest::Client, worker: &str, ek_pub: &[u8; 32]) -> Result<Route, CliError> {
    let url = format!("{}/route?ek={}", worker.trim_end_matches('/'), b64u_encode(ek_pub));
    let res = http.get(url).send().await.map_err(|e| CliError::Route(e.to_string()))?;
    let status = res.status();
    let text = res.text().await.map_err(|e| CliError::Route(e.to_string()))?;
    if !status.is_success() {
        return Err(CliError::Route(format!("{status}: {text}")));
    }
    serde_json::from_str(&text).map_err(|e| CliError::Route(format!("unexpected response: {e}")))
}

fn admission(node: &str, reason: impl Into<String>) -> CliError {
    CliError::Admission { node: node.to_owned(), reason: reason.into() }
}

async fn next_json(ws: &mut NodeSocket, node: &str, wait: Duration) -> Result<Value, CliError> {
    loop {
        let frame = tokio::time::timeout(wait, ws.next())
            .await
            .map_err(|_| admission(node, "timeout"))?
            .ok_or_else(|| admission(node, "connection closed"))?
            .map_err(|e| admission(node, e.to_string()))?;
        match frame {
            Message::Text(t) => return serde_json::from_str(t.as_str()).map_err(|e| admission(node, e.to_string())),
            Message::Close(f) => {
                let why = f.map_or_else(|| "closed".to_owned(), |f| format!("closed {} {}", u16::from(f.code), f.reason));
                return Err(admission(node, why));
            }
            _ => {}
        }
    }
}

/// Admission na jednom nodu: přijmout `hello`, odeslat `auth`, počkat na `ready` (spec §8.1).
pub async fn admit(url: &str, node: &RouteNode, ek: &SigningKey) -> Result<NodeSocket, CliError> {
    let wait = Duration::from_secs(15);
    let (mut ws, _) = tokio_tungstenite::connect_async(url).await.map_err(|e| admission(&node.node_id, e.to_string()))?;
    let hello = next_json(&mut ws, &node.node_id, wait).await?;
    if hello["t"] != "hello" || hello["node_id"] != node.node_id.as_str() {
        return Err(admission(&node.node_id, "unexpected hello"));
    }
    let challenge = hello["challenge"].as_str().and_then(b64u_decode_array::<32>).ok_or_else(|| admission(&node.node_id, "bad challenge"))?;
    let mut payload = challenge.to_vec();
    payload.extend_from_slice(node.node_id.as_bytes());
    let sig = sign(ek, DOMAIN_CLIENT_AUTH, &payload);
    let auth = json!({"t": "auth", "ticket": node.ticket, "sig": b64u_encode(&sig)});
    ws.send(Message::text(auth.to_string())).await.map_err(|e| admission(&node.node_id, e.to_string()))?;
    let answer = next_json(&mut ws, &node.node_id, wait).await?;
    match answer["t"].as_str() {
        Some("ready") => Ok(ws),
        _ => Err(admission(&node.node_id, answer["reason"].as_str().unwrap_or("rejected"))),
    }
}

/// Zkouší nody v pořadí z `/route`, první, který klienta přijme, vyhrává (failover).
/// `url_for` převádí hostname na adresu WebSocketu (výchozí `wss://<hostname>/client`).
pub async fn connect_any(route: &Route, ek: &SigningKey, url_for: impl Fn(&str) -> String) -> Result<(RouteNode, NodeSocket), CliError> {
    let mut failures = Vec::new();
    for node in &route.nodes {
        match admit(&url_for(&node.hostname), node, ek).await {
            Ok(ws) => return Ok((node.clone(), ws)),
            Err(e) => failures.push(e.to_string()),
        }
    }
    Err(CliError::AllFailed(failures.join("; ")))
}

pub fn default_url(hostname: &str) -> String {
    format!("wss://{hostname}/client")
}

/// Jeden požadavek a odpověď na již přijatém spojení (spec §13.3). Odpověď se páruje podle `rid`.
pub async fn rpc(ws: &mut NodeSocket, mut request: Value) -> Result<Value, CliError> {
    let mut raw = [0u8; 8];
    rand::rngs::OsRng.fill_bytes(&mut raw);
    let rid = b64u_encode(&raw);
    request["rid"] = json!(rid);
    ws.send(Message::text(request.to_string())).await.map_err(|e| CliError::Connection(e.to_string()))?;
    let wait = Duration::from_secs(30);
    loop {
        let frame = tokio::time::timeout(wait, ws.next())
            .await
            .map_err(|_| CliError::Connection("timeout".into()))?
            .ok_or_else(|| CliError::Connection("connection closed".into()))?
            .map_err(|e| CliError::Connection(e.to_string()))?;
        let Message::Text(text) = frame else {
            if matches!(frame, Message::Close(_)) {
                return Err(CliError::Connection("connection closed".into()));
            }
            continue;
        };
        let reply: Value = serde_json::from_str(text.as_str()).map_err(|e| CliError::Connection(e.to_string()))?;
        if reply["rid"] != rid.as_str() {
            continue;
        }
        if reply["t"] == "error" {
            return Err(CliError::Node {
                code: reply["code"].as_str().unwrap_or("error").to_owned(),
                reason: reply["reason"].as_str().unwrap_or("").to_owned(),
            });
        }
        return Ok(reply);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Initiator,
    Responder,
}

impl Role {
    fn sending(self) -> Direction {
        match self {
            Self::Initiator => Direction::InitiatorToResponder,
            Self::Responder => Direction::ResponderToInitiator,
        }
    }
}

/// Stav konverzace jedné dvojice: sdílené tajemství a oba ratchety. Ukládá se do souboru.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub secret: [u8; 32],
    pub role: Role,
    tx: Sender,
    rx: Receiver,
}

/// Zpráva zašifrovaná a připravená k odeslání. Opakované odeslání téhož objektu je idempotentní.
#[derive(Debug, Clone)]
pub struct Prepared {
    pub id: [u8; 16],
    pub mailbox_id: [u8; 32],
    pub ct: String,
}

#[derive(Debug, Default)]
pub struct Received {
    pub messages: Vec<Vec<u8>>,
    /// Zprávy, které se nepodařilo dešifrovat (podvržené, přehrané, desync). Zůstanou v mailboxu do vypršení.
    pub rejected: usize,
}

impl Session {
    pub fn new(secret: [u8; 32], role: Role) -> Self {
        Self {
            secret,
            role,
            tx: Sender::new(&secret, role.sending()),
            rx: Receiver::new(&secret, role.sending().opposite()),
        }
    }

    pub fn load(path: &std::path::Path) -> Result<Self, CliError> {
        let text = std::fs::read_to_string(path).map_err(|e| CliError::Session(format!("{}: {e}", path.display())))?;
        serde_json::from_str(&text).map_err(|e| CliError::Session(format!("{}: {e}", path.display())))
    }

    /// Zapíše soubor s právy 0600 (obsahuje klíče).
    pub fn save(&self, path: &std::path::Path) -> Result<(), CliError> {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let text = serde_json::to_string_pretty(self).map_err(|e| CliError::Session(e.to_string()))?;
        let tmp = path.with_extension("tmp");
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&tmp)
            .map_err(|e| CliError::Session(format!("{}: {e}", tmp.display())))?;
        file.write_all(text.as_bytes()).map_err(|e| CliError::Session(e.to_string()))?;
        std::fs::rename(&tmp, path).map_err(|e| CliError::Session(format!("{}: {e}", path.display())))
    }

    /// Zašifruje zprávu a posune řetěz. Stav je nutné uložit dřív, než se zpráva odešle, jinak by se po pádu
    /// stejné číslo zprávy použilo podruhé s jiným obsahem.
    pub fn prepare(&mut self, plaintext: &[u8]) -> Result<Prepared, CliError> {
        let mut id = [0u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut id);
        let ct = self.tx.seal(&id, plaintext).map_err(|e| CliError::Session(e.to_string()))?;
        Ok(Prepared { id, mailbox_id: self.tx.mailbox_id(), ct })
    }

    pub fn inbox(&self) -> [u8; 32] {
        self.rx.mailbox_id()
    }
}

/// Odešle připravenou zprávu. Vrací počet anchorů, které ji potvrdily.
pub async fn transmit(ws: &mut NodeSocket, msg: &Prepared) -> Result<u64, CliError> {
    transmit_with_push(ws, msg, None).await
}

/// Jako `transmit`; s `push_id` příjemce anchory po uložení zprávy pošlou wake (spec §13.6).
pub async fn transmit_with_push(ws: &mut NodeSocket, msg: &Prepared, push_id: Option<&str>) -> Result<u64, CliError> {
    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let mut request = json!({"t": "send", "id": b64u_encode(&msg.id), "mailbox_id": b64u_encode(&msg.mailbox_id), "ts": ts, "ct": msg.ct});
    if let Some(push_id) = push_id {
        request["push_id"] = json!(push_id);
    }
    let reply = rpc(ws, request).await?;
    Ok(reply["acks"].as_u64().unwrap_or(0))
}

/// Stáhne zprávy od protistrany, dešifruje je v pořadí a potvrdí doručení. `session` je po volání nutné uložit.
pub async fn receive(ws: &mut NodeSocket, session: &mut Session) -> Result<Received, CliError> {
    let mailbox = session.inbox();
    let reply = rpc(ws, json!({"t": "fetch", "mailbox_id": b64u_encode(&mailbox), "limit": 500})).await?;
    let mut out = Received::default();
    let mut opened = Vec::new();
    let mut delivered = Vec::new();
    for m in reply["messages"].as_array().map_or(&[][..], Vec::as_slice) {
        let parsed = m["id"].as_str().and_then(b64u_decode_array::<16>).zip(m["ct"].as_str());
        match parsed.map(|(id, ct)| (id, session.rx.open_numbered(&id, ct))) {
            Some((id, Ok((n, plain)))) => {
                opened.push((n, plain));
                delivered.push(b64u_encode(&id));
            }
            _ => out.rejected += 1,
        }
    }
    // pořadí určuje ratchet, ne `ts` (sekundová přesnost)
    opened.sort_by_key(|(n, _)| *n);
    out.messages = opened.into_iter().map(|(_, plain)| plain).collect();
    if !delivered.is_empty() {
        rpc(ws, json!({"t": "ack", "mailbox_id": b64u_encode(&mailbox), "ids": delivered})).await?;
    }
    Ok(out)
}
