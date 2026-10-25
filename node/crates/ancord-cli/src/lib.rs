//! Minimální klient: `GET /route`, zabezpečené připojení na node s ticketem a failover (`spec/PROTOCOL.md` §8, §8.1, §14.1).

use std::time::Duration;

use ancord_node::channel::{self, Channel, Ephemeral};
use ancord_node::encoding::{b64u_decode_array, b64u_encode};
use ancord_node::identity::node_id_of;
use ancord_node::ratchet::{Direction, Receiver, Sender};
use ancord_node::sign::{self, sign, DOMAIN_CHAN_CLIENT, DOMAIN_CHAN_NODE};
use ancord_node::ticket::verify_ticket;
use ed25519_dalek::SigningKey;
use futures_util::{SinkExt, StreamExt};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

type RawSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Spojení s nodem po navázání šifrovaného kanálu: každý rámec je jeden šifrovaný JSON.
/// Obsah (klíče kanálu) se nikdy nevypisuje.
impl std::fmt::Debug for NodeSocket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NodeSocket")
    }
}

pub struct NodeSocket {
    ws: RawSocket,
    chan: Channel,
}

impl NodeSocket {
    pub async fn send_json(&mut self, value: &Value) -> Result<(), CliError> {
        let frame = self.chan.send.seal(value.to_string().as_bytes()).map_err(|e| CliError::Connection(e.to_string()))?;
        self.ws.send(Message::Binary(frame.into())).await.map_err(|e| CliError::Connection(e.to_string()))
    }

    /// Další zpráva od nodu. Rámec, který nejde ověřit, je chyba spojení (nikdy se nepřeskakuje).
    pub async fn recv_json(&mut self) -> Result<Value, CliError> {
        loop {
            let frame = self
                .ws
                .next()
                .await
                .ok_or_else(|| CliError::Connection("connection closed".into()))?
                .map_err(|e| CliError::Connection(e.to_string()))?;
            match frame {
                Message::Binary(b) => {
                    let plain = self.chan.recv.open(&b).map_err(|e| CliError::Connection(e.to_string()))?;
                    return serde_json::from_slice(&plain).map_err(|e| CliError::Connection(e.to_string()));
                }
                Message::Close(_) => return Err(CliError::Connection("connection closed".into())),
                Message::Text(_) => return Err(CliError::Connection("unexpected plaintext frame".into())),
                _ => {}
            }
        }
    }

    pub async fn recv_json_timeout(&mut self, wait: Duration) -> Result<Option<Value>, CliError> {
        match tokio::time::timeout(wait, self.recv_json()).await {
            Err(_) => Ok(None),
            Ok(r) => r.map(Some),
        }
    }

    pub async fn close(&mut self) {
        let _ = self.ws.close(None).await;
    }
}

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

async fn next_text(ws: &mut RawSocket, node: &str, wait: Duration) -> Result<Value, CliError> {
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

/// Admission na jednom nodu (`spec/PROTOCOL.md` §8.1 a §14.1): ověřit ticket klíčem Workeru, `hello`, výměna klíčů
/// s ověřením identity nodu, šifrovaný `auth`, `ready`.
///
/// `worker_key` musí klient znát předem (připnutý mimo síť): jinak by Cloudflare mohl podvrhnout ticket i node.
pub async fn admit(url: &str, node: &RouteNode, ek: &SigningKey, worker_key: &[u8; 32]) -> Result<NodeSocket, CliError> {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    // ticket musí být od Workeru, pro tento node a pro náš klíč: jinak nás někdo přesměrovává na cizí node
    let body = verify_ticket(worker_key, &node.ticket, &node.node_id, now).map_err(|e| admission(&node.node_id, format!("untrusted route: {e}")))?;
    if body.ek != b64u_encode(ek.verifying_key().as_bytes()) {
        return Err(admission(&node.node_id, "untrusted route: ticket is for another key"));
    }
    admit_unchecked(url, node, ek).await
}

/// Jako `admit`, ale ticket neověřuje a o jeho platnosti rozhodne až node. Jen pro testy odmítnutí na straně nodu:
/// skutečný klient musí volat `admit`. Identitu nodu ověřuje i tahle varianta.
pub async fn admit_unchecked(url: &str, node: &RouteNode, ek: &SigningKey) -> Result<NodeSocket, CliError> {
    let wait = Duration::from_secs(15);

    let (mut ws, _) = tokio_tungstenite::connect_async(url).await.map_err(|e| admission(&node.node_id, e.to_string()))?;
    let hello = next_text(&mut ws, &node.node_id, wait).await?;
    if hello["t"] != "hello" || hello["node_id"] != node.node_id.as_str() {
        return Err(admission(&node.node_id, "unexpected hello"));
    }
    let challenge = hello["challenge"].as_str().and_then(b64u_decode_array::<32>).ok_or_else(|| admission(&node.node_id, "bad challenge"))?;

    let ce = Ephemeral::generate();
    let ce_pub = ce.public();
    ws.send(Message::text(json!({"t": "init", "ce": b64u_encode(&ce_pub)}).to_string())).await.map_err(|e| admission(&node.node_id, e.to_string()))?;
    let accept = next_text(&mut ws, &node.node_id, wait).await?;
    let (Some(ne), Some(npk), Some(sig)) = (
        accept["ne"].as_str().and_then(b64u_decode_array::<32>),
        accept["npk"].as_str().and_then(b64u_decode_array::<32>),
        accept["sig"].as_str().and_then(b64u_decode_array::<64>),
    ) else {
        return Err(admission(&node.node_id, accept["reason"].as_str().unwrap_or("malformed accept")));
    };
    // identita nodu: klíč musí odpovídat `node_id` z podepsaného ticketu a podpis musí pokrýt celý přepis handshaku
    if node_id_of(&npk) != node.node_id {
        return Err(admission(&node.node_id, "node key does not match node_id"));
    }
    let th = channel::client_transcript(&challenge, &ce_pub, &ne, &npk, &node.node_id);
    if !sign::verify(&npk, DOMAIN_CHAN_NODE, &th, &sig) {
        return Err(admission(&node.node_id, "node failed to prove its identity"));
    }
    let shared = ce.diffie_hellman(&ne).map_err(|e| admission(&node.node_id, e.to_string()))?;
    let mut conn = NodeSocket { ws, chan: Channel::for_client(&shared, &th) };

    let auth = json!({"t": "auth", "ticket": node.ticket, "sig": b64u_encode(&sign(ek, DOMAIN_CHAN_CLIENT, &th))});
    conn.send_json(&auth).await.map_err(|e| admission(&node.node_id, e.to_string()))?;
    let answer = match conn.recv_json_timeout(wait).await {
        Ok(Some(v)) => v,
        Ok(None) => return Err(admission(&node.node_id, "timeout")),
        Err(e) => return Err(admission(&node.node_id, e.to_string())),
    };
    match answer["t"].as_str() {
        Some("ready") => Ok(conn),
        _ => Err(admission(&node.node_id, answer["reason"].as_str().unwrap_or("rejected"))),
    }
}

/// Zkouší nody v pořadí z `/route`, první, který klienta přijme, vyhrává (failover).
/// `url_for` převádí hostname na adresu WebSocketu (výchozí `wss://<hostname>/client`).
pub async fn connect_any(
    route: &Route,
    ek: &SigningKey,
    worker_key: &[u8; 32],
    url_for: impl Fn(&str) -> String,
) -> Result<(RouteNode, NodeSocket), CliError> {
    let mut failures = Vec::new();
    for node in &route.nodes {
        match admit(&url_for(&node.hostname), node, ek, worker_key).await {
            Ok(ws) => return Ok((node.clone(), ws)),
            Err(e) => failures.push(e.to_string()),
        }
    }
    Err(CliError::AllFailed(failures.join("; ")))
}

pub fn default_url(hostname: &str) -> String {
    format!("wss://{hostname}/client")
}

/// Jeden požadavek a odpověď na již přijatém spojení (spec §13.3). Odpověď se páruje podle `rid`;
/// zprávy `message` přeposlané anchorem mezitím zůstávají v mailboxu a vyzvednou se přes `fetch`.
pub async fn rpc(ws: &mut NodeSocket, mut request: Value) -> Result<Value, CliError> {
    let mut raw = [0u8; 8];
    rand::rngs::OsRng.fill_bytes(&mut raw);
    let rid = b64u_encode(&raw);
    request["rid"] = json!(rid);
    ws.send_json(&request).await?;
    let wait = Duration::from_secs(30);
    loop {
        let reply = ws.recv_json_timeout(wait).await?.ok_or_else(|| CliError::Connection("timeout".into()))?;
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

/// Přihlásí odběr příchozího mailboxu: gateway oznámí anchorům, že je příjemce online u něj (spec §13.7).
/// Po `subscribe` je nutné zavolat `receive` na zprávy, které čekaly offline.
pub async fn subscribe(ws: &mut NodeSocket, session: &Session) -> Result<(), CliError> {
    rpc(ws, json!({"t": "subscribe", "mailbox_id": b64u_encode(&session.inbox())})).await.map(|_| ())
}

/// Počká na zprávu, kterou anchor přeposlal na gateway, dešifruje ji a potvrdí. `None` po vypršení `wait`.
/// Duplicity a nedešifrovatelné zprávy přeskočí. `session` je po volání nutné uložit.
pub async fn next_pushed(ws: &mut NodeSocket, session: &mut Session, wait: Duration) -> Result<Option<Vec<u8>>, CliError> {
    let deadline = tokio::time::Instant::now() + wait;
    loop {
        let left = deadline.saturating_duration_since(tokio::time::Instant::now());
        let Some(push) = ws.recv_json_timeout(left).await? else { return Ok(None) };
        if push["t"] != "message" || push["mailbox_id"] != b64u_encode(&session.inbox()).as_str() {
            continue;
        }
        let Some((id, ct)) = push["id"].as_str().and_then(b64u_decode_array::<16>).zip(push["ct"].as_str()) else { continue };
        if let Ok(plain) = session.rx.open(&id, ct) {
            rpc(ws, json!({"t": "ack", "mailbox_id": b64u_encode(&session.inbox()), "ids": [b64u_encode(&id)]})).await?;
            return Ok(Some(plain));
        }
    }
}
