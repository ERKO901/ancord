//! Minimální klient: `GET /route`, zabezpečené připojení na node s ticketem a failover (`spec/PROTOCOL.md` §8, §8.1, §14.1).


use std::collections::VecDeque;
use std::time::Duration;

use crate::channel::{self, Channel, Ephemeral};
use crate::encoding::{b64u_decode_array, b64u_encode};
use crate::ticket::node_id_of;
use crate::profile::Profile;
use crate::ratchet::Ratchet;
use crate::sign::{self, sign, DOMAIN_CHAN_CLIENT, DOMAIN_CHAN_NODE};
use crate::ticket::verify_ticket;
use crate::user::{inbox_id, mailbox_push_key, FirstRatchetKey, Opened, Pairwise, UserKeys};
use crate::wake::push_id_of;
use ed25519_dalek::SigningKey;
use futures_util::{SinkExt, StreamExt};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use x25519_dalek::StaticSecret;

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
    /// Rámce bez `rid` (`message`, `typing`), které gateway poslala, zatímco `rpc` čekal na odpověď.
    queued: VecDeque<Value>,
}

impl NodeSocket {
    pub async fn send_json(&mut self, value: &Value) -> Result<(), CliError> {
        let frame = self.chan.send.seal(value.to_string().as_bytes()).map_err(|e| CliError::Connection(e.to_string()))?;
        self.ws.send(Message::Binary(frame.into())).await.map_err(|e| CliError::Connection(e.to_string()))
    }

    /// Další zpráva od nodu. Rámec, který nejde ověřit, je chyba spojení (nikdy se nepřeskakuje).
    pub async fn recv_json(&mut self) -> Result<Value, CliError> {
        if let Some(v) = self.queued.pop_front() {
            return Ok(v);
        }
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
    /// Střežená schránka chce razítko s touto obtížností (spec §15.13).
    #[error("inbox requires a proof-of-work stamp of {0} bits")]
    PowRequired(u8),
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
    // `hello` bez `v` je starý node (před verzováním): nesoulad se pozná hned, ne až po první zprávě
    match hello["v"].as_u64() {
        Some(v) if v == u64::from(crate::PROTOCOL_VERSION) => {}
        Some(v) => return Err(admission(&node.node_id, format!("protocol mismatch: node v{v}, client v{}", crate::PROTOCOL_VERSION))),
        None => return Err(admission(&node.node_id, format!("protocol mismatch: node without version, client v{}", crate::PROTOCOL_VERSION))),
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
    let mut conn = NodeSocket { ws, chan: Channel::for_client(&shared, &th), queued: VecDeque::new() };

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
/// zprávy `message` a `typing`, které gateway mezitím pošle, se zařadí a vrátí další `recv_json`.
pub async fn rpc(ws: &mut NodeSocket, mut request: Value) -> Result<Value, CliError> {
    let mut raw = [0u8; 8];
    rand::rngs::OsRng.fill_bytes(&mut raw);
    let rid = b64u_encode(&raw);
    request["rid"] = json!(rid);
    ws.send_json(&request).await?;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    let mut queued = Vec::new();
    let reply = loop {
        let left = deadline.saturating_duration_since(tokio::time::Instant::now());
        let frame = match tokio::time::timeout(left, read_frame(ws)).await {
            Err(_) => Err(CliError::Connection("timeout".into())),
            Ok(r) => r,
        };
        let frame = match frame {
            Ok(f) => f,
            Err(e) => {
                ws.queued.extend(queued);
                return Err(e);
            }
        };
        if frame["rid"] == rid.as_str() {
            break frame;
        }
        // odpověď na dřívější požadavek, který vypršel, se zahodí
        if frame.get("rid").is_none() {
            queued.push(frame);
        }
    };
    ws.queued.extend(queued);
    if reply["t"] == "error" {
        if reply["code"] == "pow_required" {
            let bits = reply["bits"].as_u64().and_then(|b| u8::try_from(b).ok()).unwrap_or(crate::guard::MAX_BITS);
            return Err(CliError::PowRequired(bits));
        }
        return Err(CliError::Node {
            code: reply["code"].as_str().unwrap_or("error").to_owned(),
            reason: reply["reason"].as_str().unwrap_or("").to_owned(),
        });
    }
    Ok(reply)
}

/// Další rámec ze sítě (frontu obchází: `rpc` musí vidět i odpovědi, které přijdou po zařazených rámcích).
async fn read_frame(ws: &mut NodeSocket) -> Result<Value, CliError> {
    let queued = std::mem::take(&mut ws.queued);
    let frame = ws.recv_json().await;
    ws.queued = queued;
    frame
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Initiator,
    Responder,
}

/// Za jak dlouho po přijaté zprávě, na kterou klient nic nenapsal, pošle prázdnou zprávu (keepalive, spec §13.2).
pub const KEEPALIVE_DELAY_S: u64 = 30;
/// Nejvýše jeden keepalive za tuto dobu v jedné konverzaci.
pub const KEEPALIVE_MIN_INTERVAL_S: u64 = 3600;

/// Stav konverzace jedné dvojice: sdílené tajemství a Double Ratchet. Ukládá se do souboru.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub secret: [u8; 32],
    pub role: Role,
    /// Adresa protistrany (b64u), je-li konverzace založená podle adresy (spec §15.4).
    #[serde(default)]
    pub peer: Option<String>,
    pub(crate) ratchet: Ratchet,
    /// Lokální historie konverzace (spec §15.7): server ji nedrží.
    #[serde(default)]
    pub history: Vec<HistoryEntry>,
    /// Od kdy čeká přijatá zpráva s DH krokem na odpověď (0 = nečeká); po `KEEPALIVE_DELAY_S` jde keepalive.
    #[serde(default)]
    unanswered_since: u64,
    #[serde(default)]
    last_keepalive: u64,
}

/// Jedna zpráva v lokální historii.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub ts: u64,
    /// `true` = odeslaná tímto klientem.
    pub outgoing: bool,
    /// `id` zprávy (b64u).
    pub id: String,
    pub text: String,
}

pub(crate) fn now_s() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// Zpráva zašifrovaná a připravená k odeslání. Opakované odeslání téhož objektu je idempotentní.
#[derive(Debug, Clone)]
pub struct Prepared {
    pub id: [u8; 16],
    pub mailbox_id: [u8; 32],
    pub ct: String,
    /// `push_id` mailboxu příjemce (spec §15.11): anchor podle něj příjemce vzbudí, není-li online.
    pub push_id: String,
}

#[derive(Debug, Default)]
pub struct Received {
    pub messages: Vec<Vec<u8>>,
    /// Zprávy, které se nepodařilo dešifrovat (podvržené, přehrané, desync). Zůstanou v mailboxu do vypršení.
    pub rejected: usize,
}

impl Session {
    /// Konverzace jen ze sdíleného tajemství (bez prekeys): první ratchet klíč responderu se z něj odvodí (spec §13.2).
    pub fn new(secret: [u8; 32], role: Role) -> Result<Self, CliError> {
        let ratchet = Ratchet::from_shared_secret(&secret, role == Role::Initiator).map_err(|e| CliError::Session(e.to_string()))?;
        Ok(Self { secret, role, peer: None, ratchet, history: Vec::new(), unanswered_since: 0, last_keepalive: 0 })
    }

    /// Konverzace ze sdíleného tajemství, které vzešlo z dohody mezi uživateli (spec §15.4).
    pub fn from_pairwise(pair: &Pairwise) -> Result<Self, CliError> {
        let (role, ratchet) = match &pair.ratchet_key {
            FirstRatchetKey::Peer(spk) => (Role::Initiator, Ratchet::initiator(&pair.secret, spk)),
            FirstRatchetKey::Own(spk_secret) => (Role::Responder, Ok(Ratchet::responder(&pair.secret, &StaticSecret::from(*spk_secret)))),
        };
        let ratchet = ratchet.map_err(|e| CliError::Session(e.to_string()))?;
        Ok(Self { secret: pair.secret, role, peer: None, ratchet, history: Vec::new(), unanswered_since: 0, last_keepalive: 0 })
    }

    /// Přijatá a dešifrovaná zpráva. Prázdný plaintext je keepalive (spec §13.2): do historie se nezapisuje a vrací se
    /// `false`. Keepalive vyvolá jen skutečná zpráva, jinak by si ho dvě strany posílaly donekonečna.
    pub(crate) fn accept(&mut self, id: &[u8; 16], plaintext: &[u8]) -> bool {
        if plaintext.is_empty() {
            return false;
        }
        if self.ratchet.needs_keepalive() && self.unanswered_since == 0 {
            self.unanswered_since = now_s();
        }
        self.record(false, id, plaintext);
        true
    }

    /// Má klient poslat keepalive? `delay` je doba od přijaté zprávy, po kterou čeká na skutečnou odpověď uživatele.
    pub fn keepalive_due(&self, now: u64, delay: u64) -> bool {
        self.ratchet.needs_keepalive()
            && self.unanswered_since > 0
            && now >= self.unanswered_since.saturating_add(delay)
            && now >= self.last_keepalive.saturating_add(KEEPALIVE_MIN_INTERVAL_S)
    }

    /// Prázdná zpráva, která protistraně předá nový ratchet klíč (spec §13.2). Odesílá se bez `push_id`: kvůli ní
    /// příjemce budit nemá smysl. Stav je nutné uložit před odesláním.
    pub fn prepare_keepalive(&mut self) -> Result<Prepared, CliError> {
        let p = self.seal(b"")?;
        self.last_keepalive = now_s();
        Ok(p)
    }

    fn record(&mut self, outgoing: bool, id: &[u8; 16], plaintext: &[u8]) {
        self.history.push(HistoryEntry { ts: now_s(), outgoing, id: b64u_encode(id), text: String::from_utf8_lossy(plaintext).into_owned() });
    }

    /// Zašifruje zprávu a posune řetěz. Stav je nutné uložit dřív, než se zpráva odešle, jinak by se po pádu
    /// stejné číslo zprávy použilo podruhé s jiným obsahem.
    pub fn prepare(&mut self, plaintext: &[u8]) -> Result<Prepared, CliError> {
        // prázdný plaintext je vyhrazený pro keepalive
        if plaintext.is_empty() {
            return Err(CliError::Session("empty message".into()));
        }
        let p = self.seal(plaintext)?;
        self.record(true, &p.id, plaintext);
        Ok(p)
    }

    fn seal(&mut self, plaintext: &[u8]) -> Result<Prepared, CliError> {
        let mut id = [0u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut id);
        let ct = self.ratchet.seal(&id, plaintext).map_err(|e| CliError::Session(e.to_string()))?;
        self.unanswered_since = 0;
        Ok(Prepared { id, mailbox_id: self.ratchet.send_mailbox_id(), ct, push_id: self.outbox_push_id() })
    }

    /// Push klíč příchozího mailboxu: pod jeho `push_id` registruje tento klient token u Workeru (spec §15.11).
    pub fn inbox_push_key(&self) -> SigningKey {
        mailbox_push_key(&self.secret, &self.inbox())
    }

    /// `push_id` mailboxu, do kterého tento klient píše (protistrana ho má zaregistrovaný).
    pub fn outbox_push_id(&self) -> String {
        push_id_of(&mailbox_push_key(&self.secret, &self.outbox()).verifying_key().to_bytes())
    }

    pub fn inbox(&self) -> [u8; 32] {
        self.ratchet.recv_mailbox_id()
    }

    /// Mailbox, do kterého tento klient píše (kam míří `typing` a `status`).
    pub fn outbox(&self) -> [u8; 32] {
        self.ratchet.send_mailbox_id()
    }
}

/// Pošle keepalive, je-li čas (`Session::keepalive_due`). Vrací `true`, když odešel. `save` uloží stav před odesláním.
pub async fn keepalive_if_due(
    ws: &mut NodeSocket,
    session: &mut Session,
    delay: u64,
    save: impl FnOnce(&Session) -> Result<(), CliError>,
) -> Result<bool, CliError> {
    if !session.keepalive_due(now_s(), delay) {
        return Ok(false);
    }
    let prepared = session.prepare_keepalive()?;
    save(session)?;
    transmit_with_push(ws, &prepared, None).await?;
    Ok(true)
}

/// Odešle připravenou zprávu. Vrací počet anchorů, které ji potvrdily. Není-li příjemce online, anchory ho vzbudí
/// pushem (spec §13.6).
pub async fn transmit(ws: &mut NodeSocket, msg: &Prepared) -> Result<u64, CliError> {
    transmit_with_push(ws, msg, Some(&msg.push_id)).await
}

/// Jako `transmit` s jiným nebo žádným `push_id`.
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
        match parsed.map(|(id, ct)| (id, session.ratchet.open_numbered(&id, ct))) {
            Some((id, Ok((n, plain)))) => {
                opened.push((n, id, plain));
                delivered.push(b64u_encode(&id));
            }
            _ => out.rejected += 1,
        }
    }
    // pořadí určuje `seq` z hlavičky ratchetu, ne `ts` (sekundová přesnost); keepalive se nezobrazuje
    opened.sort_by_key(|(n, _, _)| *n);
    opened.retain(|(_, id, plain)| session.accept(id, plain));
    out.messages = opened.into_iter().map(|(_, _, plain)| plain).collect();
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

/// Událost, kterou gateway posílá po `subscribe`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// Dešifrovaná a potvrzená zpráva.
    Message(Vec<u8>),
    /// Protistrana začala (`true`) nebo přestala psát.
    Typing(bool),
}

/// Počká na událost od gateway: zprávu (anchor ji přeposlal; dešifruje se a potvrdí) nebo signál psaní.
/// `None` po vypršení `wait`. Duplicity a nedešifrovatelné zprávy přeskočí. `session` je po volání nutné uložit.
pub async fn next_event(ws: &mut NodeSocket, session: &mut Session, wait: Duration) -> Result<Option<Event>, CliError> {
    let deadline = tokio::time::Instant::now() + wait;
    loop {
        let left = deadline.saturating_duration_since(tokio::time::Instant::now());
        let Some(push) = ws.recv_json_timeout(left).await? else { return Ok(None) };
        match push["t"].as_str() {
            // psát smí jen protistrana, takže zajímá jen signál z našeho příchozího mailboxu
            Some("typing") if push["mailbox_id"] == b64u_encode(&session.inbox()).as_str() => {
                return Ok(Some(Event::Typing(push["on"].as_bool().unwrap_or(false))));
            }
            Some("message") if push["mailbox_id"] == b64u_encode(&session.inbox()).as_str() => {
                let Some((id, ct)) = push["id"].as_str().and_then(b64u_decode_array::<16>).zip(push["ct"].as_str()) else { continue };
                if let Ok(plain) = session.ratchet.open(&id, ct) {
                    rpc(ws, json!({"t": "ack", "mailbox_id": b64u_encode(&session.inbox()), "ids": [b64u_encode(&id)]})).await?;
                    if session.accept(&id, &plain) {
                        return Ok(Some(Event::Message(plain)));
                    }
                }
            }
            _ => {}
        }
    }
}

/// Jako `next_event`, ale vrací jen zprávy (signály psaní zahazuje).
pub async fn next_pushed(ws: &mut NodeSocket, session: &mut Session, wait: Duration) -> Result<Option<Vec<u8>>, CliError> {
    let deadline = tokio::time::Instant::now() + wait;
    loop {
        let left = deadline.saturating_duration_since(tokio::time::Instant::now());
        match next_event(ws, session, left).await? {
            Some(Event::Message(m)) => return Ok(Some(m)),
            Some(Event::Typing(_)) => {}
            None => return Ok(None),
        }
    }
}

// --- uživatel (spec §15) -----------------------------------------------------------------------------------

/// Adresa uživatele pro předání protistraně (QR kód nebo text): `ancord:<pk>` (spec §15.10).
pub fn address(keys: &UserKeys) -> String {
    crate::user::address_uri(&keys.pk())
}

/// Adresa protistrany z textu (`ancord:<pk>` nebo holé `pk`).
pub fn parse_address(text: &str) -> Result<[u8; 32], CliError> {
    crate::user::parse_address(text).ok_or_else(|| CliError::Session("not an ancord address (expected ancord:<public key>)".into()))
}

/// Zaregistruje push token u Workeru pod `push_id` daného klíče (spec §10.1). `ts` musí s každou registrací růst.
pub async fn register_push(http: &reqwest::Client, worker: &str, key: &SigningKey, platform: &str, token: &str, ts: u64) -> Result<String, CliError> {
    let push_pubkey = key.verifying_key().to_bytes();
    let push_id = push_id_of(&push_pubkey);
    let payload = crate::canonical::canonical_bytes(&json!({"op": "register", "platform": platform, "push_id": push_id, "token": token, "ts": ts}))
        .map_err(|e| CliError::Session(e.to_string()))?;
    let body = json!({
        "push_pubkey": b64u_encode(&push_pubkey), "platform": platform, "token": token, "ts": ts,
        "sig": b64u_encode(&sign(key, sign::DOMAIN_PUSH_REG, &payload)),
    });
    let url = format!("{}/push/register", worker.trim_end_matches('/'));
    let res = http.post(url).json(&body).send().await.map_err(|e| CliError::Route(e.to_string()))?;
    let status = res.status();
    if !status.is_success() {
        return Err(CliError::Route(format!("push registration: {status}: {}", res.text().await.unwrap_or_default())));
    }
    Ok(push_id)
}

/// Publikuje profil uživatele na anchory. Vrací počet potvrzení. `pow_bits` musí odpovídat požadavku nodů.
/// Klíče je nutné uložit **před** voláním: profil obsahuje prekeys, jejichž tajné části musí přežít restart.
pub async fn publish_profile(ws: &mut NodeSocket, keys: &UserKeys, pow_bits: u32) -> Result<u64, CliError> {
    // hashcash je výpočetně náročný, nesmí blokovat reaktor
    let ts = now_s();
    let state = keys.to_json();
    let profile = tokio::task::spawn_blocking(move || UserKeys::from_json(&state).map(|k| k.profile(ts, pow_bits)))
        .await
        .map_err(|e| CliError::Connection(e.to_string()))?
        .ok_or_else(|| CliError::Session("malformed keys".into()))?;
    let mut request = profile.to_json();
    request["t"] = json!("profile_put");
    Ok(rpc(ws, request).await?.get("acks").and_then(Value::as_u64).unwrap_or(0))
}

/// Obtížnost razítka, kterou klient chce pro svou schránku (spec §15.13). Odesílatel ji zaplatí jednou za konverzaci.
pub const INBOX_POW_BITS: u8 = 20;

/// Zapne stráž schránky na jejích anchorech (spec §15.13): zápis bez razítka s `bits` pak anchory odmítnou.
pub async fn publish_guard(ws: &mut NodeSocket, keys: &UserKeys, bits: u8) -> Result<u64, CliError> {
    let mut request = keys.inbox_guard(bits, now_s()).to_json();
    request["t"] = json!("inbox_guard");
    Ok(rpc(ws, request).await?.get("acks").and_then(Value::as_u64).unwrap_or(0))
}

/// `ct` do schránky: razítko s obtížností `bits` a obálka (spec §15.13). Výpočet neblokuje reaktor.
async fn stamped(mailbox_id: [u8; 32], id: [u8; 16], envelope: Vec<u8>, bits: u8) -> Result<Vec<u8>, CliError> {
    tokio::task::spawn_blocking(move || crate::guard::stamp(&mailbox_id, &id, &envelope, bits))
        .await
        .map_err(|e| CliError::Connection(e.to_string()))
}

/// Najde profil podle adresy a ověří podpis i to, že patří právě této adrese.
pub async fn lookup_profile(ws: &mut NodeSocket, pk: &[u8; 32]) -> Result<Profile, CliError> {
    let reply = rpc(ws, json!({"t": "profile_get", "pk": b64u_encode(pk)})).await?;
    let profile = Profile::from_json(&reply).ok_or_else(|| CliError::Session("malformed profile".into()))?;
    if !crate::profile::is_for(&profile, pk) {
        return Err(CliError::Session("profile does not belong to the requested address".into()));
    }
    Ok(profile)
}

/// Dá protistraně vědět, že píšeme (`on`) nebo přestali jsme (spec §15.5).
pub async fn send_typing(ws: &mut NodeSocket, session: &Session, on: bool) -> Result<(), CliError> {
    rpc(ws, json!({"t": "typing", "mailbox_id": b64u_encode(&session.outbox()), "on": on})).await.map(|_| ())
}

/// Stav odeslaných zpráv: `pending`, `delivered` nebo `unknown` (spec §15.6), ve stejném pořadí jako `ids`.
pub async fn message_status(ws: &mut NodeSocket, session: &Session, ids: &[[u8; 16]]) -> Result<Vec<String>, CliError> {
    let list: Vec<String> = ids.iter().map(|i| b64u_encode(i)).collect();
    let reply = rpc(ws, json!({"t": "status", "mailbox_id": b64u_encode(&session.outbox()), "ids": list})).await?;
    Ok(reply["states"].as_array().map_or_else(Vec::new, |a| a.iter().filter_map(|v| v.as_str().map(str::to_owned)).collect()))
}

/// Přihlásí odběr více příchozích mailboxů najednou (spec §15.8).
pub async fn subscribe_many(ws: &mut NodeSocket, sessions: &[&Session]) -> Result<(), CliError> {
    let ids: Vec<String> = sessions.iter().map(|s| b64u_encode(&s.inbox())).collect();
    rpc(ws, json!({"t": "subscribe", "mailbox_ids": ids})).await.map(|_| ())
}

/// Zahájí konverzaci podle adresy (spec §15.4): najde a ověří profil, provede X3DH a pošle příjemci obálku do jeho
/// schránky. Vrací konverzaci; zprávy lze posílat hned, čekají v mailboxu, dokud příjemce obálku nezpracuje.
pub async fn start_conversation(ws: &mut NodeSocket, keys: &UserKeys, peer: &[u8; 32]) -> Result<Session, CliError> {
    let profile = lookup_profile(ws, peer).await?;
    let initiation = keys.initiate(&profile, now_s()).map_err(|e| CliError::Session(e.to_string()))?;
    let mut id = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut id);
    let inbox = inbox_id(peer);
    // obtížnost razítka klient předem nezná: nestřežená schránka vezme nulové, jinak ji anchory řeknou (spec §15.13)
    let mut bits = 0;
    loop {
        let ct = stamped(inbox, id, initiation.envelope.clone(), bits).await?;
        // `push_id` schránky z profilu: offline příjemce se o nové konverzaci dozví pushem (spec §15.11)
        let request = json!({
            "t": "send", "id": b64u_encode(&id), "mailbox_id": b64u_encode(&inbox), "ts": now_s(),
            "ct": b64u_encode(&ct), "push_id": profile.push_id,
        });
        match rpc(ws, request).await {
            Ok(_) => break,
            Err(CliError::PowRequired(want)) if want > bits && want <= crate::guard::MAX_BITS => bits = want,
            Err(e) => return Err(e),
        }
    }
    let mut session = Session::from_pairwise(&initiation.pairwise)?;
    session.peer = Some(b64u_encode(peer));
    Ok(session)
}

/// Nová konverzace, kterou si protistrana vyžádala.
pub struct Accepted {
    pub peer: [u8; 32],
    pub session: Session,
}

/// Zpracované obálky ze schránky.
#[derive(Default)]
pub struct InboxBatch {
    pub accepted: Vec<Accepted>,
    /// Obálky, které se mají potvrdit (poškozené nebo zpracované). Potvrdit až po uložení klíčů a konverzací.
    pub done: Vec<String>,
    /// Obálky, které nešlo zpracovat teď (např. profil odesílatele zatím není k nalezení), zůstávají ve schránce.
    pub postponed: usize,
}

/// Vyzvedne úvodní obálky ze schránky uživatele a dokončí dohodu. Použité jednorázové prekeys se v `keys` smažou:
/// volající musí klíče **uložit** a teprve pak zavolat `ack_inbox`.
pub async fn process_inbox(ws: &mut NodeSocket, keys: &mut UserKeys) -> Result<InboxBatch, CliError> {
    let inbox = inbox_id(&keys.pk());
    let reply = rpc(ws, json!({"t": "fetch", "mailbox_id": b64u_encode(&inbox), "limit": 500})).await?;
    let mut batch = InboxBatch::default();
    let mut seen = std::collections::HashSet::new();
    for m in reply["messages"].as_array().map_or(&[][..], Vec::as_slice) {
        let (Some(id), Some(ct)) = (m["id"].as_str(), m["ct"].as_str().and_then(crate::encoding::b64u_decode)) else { continue };
        // razítko (spec §15.13) ověřily anchory, klientovi stačí obálka za ním
        let env = crate::guard::strip_stamp(&ct).unwrap_or_default();
        let opened: Opened = match keys.open_envelope(env) {
            Ok(o) => o,
            // cizí, poškozené nebo zastaralé: nikdy nebude platné, uvolní se místo ve schránce
            Err(_) => {
                batch.done.push(id.to_owned());
                continue;
            }
        };
        if !seen.insert(opened.peer_pk) {
            batch.done.push(id.to_owned());
            continue;
        }
        let Ok(profile) = lookup_profile(ws, &opened.peer_pk).await else {
            batch.postponed += 1;
            continue;
        };
        // neplatnou obálku (i s nepoužitelným ratchet klíčem) potvrdíme a zahodíme, aby neblokovala schránku
        if let Some(mut session) = keys.accept(&opened, &profile).ok().and_then(|pair| Session::from_pairwise(&pair).ok()) {
            session.peer = Some(b64u_encode(&opened.peer_pk));
            batch.accepted.push(Accepted { peer: opened.peer_pk, session });
        }
        batch.done.push(id.to_owned());
    }
    Ok(batch)
}

/// Potvrdí zpracované obálky (uvolní schránku).
pub async fn ack_inbox(ws: &mut NodeSocket, keys: &UserKeys, done: &[String]) -> Result<(), CliError> {
    if done.is_empty() {
        return Ok(());
    }
    rpc(ws, json!({"t": "ack", "mailbox_id": b64u_encode(&inbox_id(&keys.pk())), "ids": done})).await.map(|_| ())
}

/// Bezpečnostní číslo s protistranou k porovnání mimo síť (spec §15.2).
pub fn safety(keys: &UserKeys, peer: &[u8; 32]) -> String {
    crate::user::safety_number(&keys.pk(), peer)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    fn pair() -> (Session, Session) {
        (Session::new([5; 32], Role::Initiator).unwrap(), Session::new([5; 32], Role::Responder).unwrap())
    }

    fn deliver(from: &mut Session, to: &mut Session, text: &[u8]) -> bool {
        let p = if text.is_empty() { from.prepare_keepalive().unwrap() } else { from.prepare(text).unwrap() };
        let plain = to.ratchet.open(&p.id, &p.ct).unwrap();
        to.accept(&p.id, &plain)
    }

    #[test]
    fn keepalive_after_real_message_only_and_not_shown() {
        let (mut a, mut b) = pair();
        assert!(a.prepare(b"").is_err(), "empty plaintext is reserved for keepalive");
        assert!(deliver(&mut a, &mut b, b"ahoj"));
        let now = now_s();
        assert!(!b.keepalive_due(now, KEEPALIVE_DELAY_S), "waits for a real reply first");
        assert!(b.keepalive_due(now + KEEPALIVE_DELAY_S, KEEPALIVE_DELAY_S));
        // keepalive dorazí, do historie se nezapíše a sám další keepalive nevyvolá
        assert!(!deliver(&mut b, &mut a, b""));
        assert_eq!(a.history.len(), 1);
        assert!(!a.keepalive_due(now + 10 * KEEPALIVE_MIN_INTERVAL_S, 0));
        assert!(!b.keepalive_due(now + 10 * KEEPALIVE_MIN_INTERVAL_S, 0));
        // další zpráva Alice jde novým řetězem; Bob smí poslat další keepalive až po minimálním intervalu
        assert!(deliver(&mut a, &mut b, "druhá".as_bytes()));
        assert!(!b.keepalive_due(now + KEEPALIVE_DELAY_S, KEEPALIVE_DELAY_S));
        assert!(b.keepalive_due(now + KEEPALIVE_MIN_INTERVAL_S + KEEPALIVE_DELAY_S, KEEPALIVE_DELAY_S));
        // skutečná odpověď keepalive zruší
        assert!(deliver(&mut b, &mut a, "odpověď".as_bytes()));
        assert!(!b.keepalive_due(now + 10 * KEEPALIVE_MIN_INTERVAL_S, 0));
    }
}
