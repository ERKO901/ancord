//! Admission klientů: WebSocket `/client` s tiketem od Workeru (`spec/PROTOCOL.md` §8.1).

use std::collections::HashSet;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use serde_json::{json, Value};
use tokio::sync::mpsc;

use crate::channel::{self, Channel, Ephemeral};
use crate::data::{self, DataError};
use crate::encoding::{b64u_decode_array, b64u_encode};
use crate::peer;
use crate::presence;
use crate::sign::{self, DOMAIN_CHAN_CLIENT, DOMAIN_CHAN_NODE};
use crate::state::NodeState;
use crate::ticket::{verify_ticket, TicketBody, TicketError};
use crate::time::unix_now;
use crate::wake;

const MAX_MESSAGE_BYTES: usize = 64 * 1024;
pub const CLOSE_UNAUTHORIZED: u16 = 4401;
pub const CLOSE_AUTH_TIMEOUT: u16 = 4408;
/// Rámec po navázání kanálu nešel ověřit nebo dešifrovat.
pub const CLOSE_BAD_FRAME: u16 = 4400;
const CLOSE_OVERLOADED: u16 = 1013;

pub async fn upgrade(State(st): State<Arc<NodeState>>, ws: WebSocketUpgrade) -> Response {
    ws.max_message_size(MAX_MESSAGE_BYTES).on_upgrade(move |socket| session(st, socket))
}

/// Počítá aktivní spojení do heartbeatu, i když relace skončí chybou.
struct ConnGuard(Arc<NodeState>);

impl ConnGuard {
    fn new(st: &Arc<NodeState>) -> Self {
        st.conns.fetch_add(1, Ordering::Relaxed);
        Self(st.clone())
    }
}

impl Drop for ConnGuard {
    fn drop(&mut self) {
        self.0.conns.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Při ukončení spojení zruší jeho odběry a presence mailboxů, které tím zůstaly bez odběratele.
struct SubscriptionGuard {
    st: Arc<NodeState>,
    conn: u64,
}

impl Drop for SubscriptionGuard {
    fn drop(&mut self) {
        let orphaned = presence::remove_conn(&self.st, self.conn);
        if orphaned.is_empty() {
            return;
        }
        let st = self.st.clone();
        tokio::spawn(async move {
            for mailbox in orphaned {
                presence::announce(&st, mailbox, false).await;
            }
        });
    }
}

async fn close(socket: &mut WebSocket, code: u16, reason: &str) {
    let _ = socket.send(Message::Close(Some(CloseFrame { code, reason: reason.to_owned().into() }))).await;
}

/// Zrušení před navázáním kanálu: obyčejná chyba a zavření (nic tajného se zatím neposlalo).
async fn reject(socket: &mut WebSocket, reason: &str) {
    let _ = socket.send(Message::text(json!({"t": "error", "reason": reason}).to_string())).await;
    close(socket, CLOSE_UNAUTHORIZED, reason).await;
}

/// WebSocket po navázání kanálu: každý rámec je jeden šifrovaný JSON (`spec/PROTOCOL.md` §14.1).
struct Secure {
    socket: WebSocket,
    chan: Channel,
}

enum Incoming {
    Json(Value),
    /// Rámec, který nejde ověřit nebo není JSON: spojení se zavře.
    Invalid,
    Closed,
}

impl Secure {
    async fn send(&mut self, value: &Value) -> bool {
        match self.chan.send.seal(value.to_string().as_bytes()) {
            Ok(frame) => self.socket.send(Message::Binary(frame.into())).await.is_ok(),
            Err(_) => false,
        }
    }

    async fn recv(&mut self) -> Incoming {
        loop {
            match self.socket.recv().await {
                Some(Ok(Message::Binary(frame))) => {
                    return match self.chan.recv.open(&frame).ok().and_then(|p| serde_json::from_slice(&p).ok()) {
                        Some(v) => Incoming::Json(v),
                        None => Incoming::Invalid,
                    };
                }
                // po navázání kanálu se nepřijímá nic nešifrovaného, ani text
                Some(Ok(Message::Text(_))) => return Incoming::Invalid,
                Some(Ok(Message::Ping(_) | Message::Pong(_))) => {}
                Some(Ok(Message::Close(_)) | Err(_)) | None => return Incoming::Closed,
            }
        }
    }

    async fn reject(&mut self, reason: &str) {
        let _ = self.send(&json!({"t": "error", "reason": reason})).await;
        close(&mut self.socket, CLOSE_UNAUTHORIZED, reason).await;
    }
}

/// Ověří `auth`: ticket, podpis klíčem `ek` přes přepis handshaku, jednorázovost. Ticket se spotřebuje až po úspěšném
/// podpisu, aby ho nemohl vypálit někdo, kdo má ticket, ale ne klíč.
fn authenticate(st: &NodeState, th: &[u8; 32], msg: &Value) -> Result<TicketBody, &'static str> {
    if msg["t"] != "auth" {
        return Err("expected auth");
    }
    let ticket = msg["ticket"].as_str().ok_or("missing ticket")?;
    let sig = msg["sig"].as_str().and_then(b64u_decode_array::<64>).ok_or("malformed signature")?;
    let now = unix_now();
    let body = verify_ticket(&st.worker_key, ticket, st.identity.node_id(), now).map_err(|e| match e {
        TicketError::Malformed => "malformed ticket",
        TicketError::BadSignature => "bad ticket signature",
        TicketError::WrongNode => "ticket issued for another node",
        TicketError::Expired => "ticket expired",
        TicketError::Replayed => "ticket already used",
    })?;
    let ek = b64u_decode_array::<32>(&body.ek).ok_or("malformed ticket")?;
    if !sign::verify(&ek, DOMAIN_CHAN_CLIENT, th, &sig) {
        return Err("invalid client signature");
    }
    st.tickets.consume(&body, now).map_err(|_| "ticket already used")?;
    Ok(body)
}

/// Čeká na textový rámec do `auth_timeout_s`. `None` = spojení skončilo nebo vypršel čas (už zavřeno).
async fn recv_plain(socket: &mut WebSocket, timeout: Duration) -> Option<Value> {
    let frame = match tokio::time::timeout(timeout, socket.recv()).await {
        Err(_) => {
            close(socket, CLOSE_AUTH_TIMEOUT, "auth timeout").await;
            return None;
        }
        Ok(None | Some(Err(_))) => return None,
        Ok(Some(Ok(m))) => m,
    };
    let Message::Text(text) = frame else {
        reject(socket, "expected text frame").await;
        return None;
    };
    let value = serde_json::from_str::<Value>(text.as_str()).ok();
    if value.is_none() {
        reject(socket, "invalid JSON").await;
    }
    value
}

/// Handshake klient ↔ node (`spec/PROTOCOL.md` §14.1) a admission. Vrací šifrovaný kanál, nebo `None`,
/// když se spojení zavřelo.
async fn establish(st: &Arc<NodeState>, mut socket: WebSocket) -> Option<Secure> {
    let timeout = Duration::from_secs(st.cfg.auth_timeout_s);
    let challenge = channel::random_array::<32>();
    let hello = json!({"t": "hello", "node_id": st.identity.node_id(), "challenge": b64u_encode(&challenge)});
    socket.send(Message::text(hello.to_string())).await.ok()?;

    let init = recv_plain(&mut socket, timeout).await?;
    let Some(ce) = (init["t"] == "init").then(|| init["ce"].as_str().and_then(b64u_decode_array::<32>)).flatten() else {
        reject(&mut socket, "expected init").await;
        return None;
    };
    let ne = Ephemeral::generate();
    let ne_pub = ne.public();
    let th = channel::client_transcript(&challenge, &ce, &ne_pub, &st.identity.pubkey(), st.identity.node_id());
    let Ok(shared) = ne.diffie_hellman(&ce) else {
        reject(&mut socket, "invalid key").await;
        return None;
    };
    let accept = json!({
        "t": "accept",
        "ne": b64u_encode(&ne_pub),
        "npk": st.identity.pubkey_b64(),
        "sig": b64u_encode(&st.identity.sign(DOMAIN_CHAN_NODE, &th)),
    });
    socket.send(Message::text(accept.to_string())).await.ok()?;

    let mut secure = Secure { socket, chan: Channel::for_node(&shared, &th) };
    let first = match tokio::time::timeout(timeout, secure.recv()).await {
        Err(_) => {
            close(&mut secure.socket, CLOSE_AUTH_TIMEOUT, "auth timeout").await;
            return None;
        }
        Ok(Incoming::Closed) => return None,
        Ok(Incoming::Invalid) => {
            close(&mut secure.socket, CLOSE_UNAUTHORIZED, "invalid frame").await;
            return None;
        }
        Ok(Incoming::Json(v)) => v,
    };
    if let Err(reason) = authenticate(st, &th, &first) {
        secure.reject(reason).await;
        return None;
    }
    secure.send(&json!({"t": "ready"})).await.then_some(secure)
}

async fn session(st: Arc<NodeState>, mut socket: WebSocket) {
    if st.conns.load(Ordering::Relaxed) >= st.cfg.max_conns {
        close(&mut socket, CLOSE_OVERLOADED, "node is full").await;
        return;
    }
    let _guard = ConnGuard::new(&st);
    let Some(mut socket) = establish(&st, socket).await else { return };

    let conn = st.next_conn.fetch_add(1, Ordering::Relaxed);
    let _subscriptions = SubscriptionGuard { st: st.clone(), conn };
    let (push_tx, mut push_rx) = mpsc::channel::<Value>(64);
    let mut subscribed: HashSet<[u8; 32]> = HashSet::new();
    let mut bucket = Bucket::new(st.cfg.client_msg_rate);
    loop {
        tokio::select! {
            Some(push) = push_rx.recv() => {
                if !socket.send(&push).await {
                    return;
                }
            }
            incoming = socket.recv() => {
                let msg = match incoming {
                    Incoming::Json(msg) => msg,
                    Incoming::Invalid => return close(&mut socket.socket, CLOSE_BAD_FRAME, "invalid frame").await,
                    Incoming::Closed => return,
                };
                let reply = if msg["t"] == "ping" {
                    json!({"t": "pong"})
                } else if !bucket.take() {
                    let mut r = json!({"t": "error", "code": "rate_limited", "reason": "too many messages"});
                    attach_rid(&mut r, &msg);
                    r
                } else if matches!(msg["t"].as_str(), Some("subscribe" | "unsubscribe")) {
                    let mut r = subscription(&st, &msg, conn, &push_tx, &mut subscribed).await.unwrap_or_else(|e| error_reply(&e, None));
                    attach_rid(&mut r, &msg);
                    r
                } else {
                    handle(&st, &msg).await
                };
                if !socket.send(&reply).await {
                    return;
                }
            }
        }
    }
}

/// `subscribe` a `unsubscribe` (`spec/PROTOCOL.md` §13.7): gateway oznámí anchorům, že je mailbox online u něj.
async fn subscription(
    st: &Arc<NodeState>,
    msg: &Value,
    conn: u64,
    tx: &mpsc::Sender<Value>,
    subscribed: &mut HashSet<[u8; 32]>,
) -> Result<Value, DataError> {
    let mailbox = data::parse_mailbox(field(msg, "mailbox_id")?)?;
    let id = b64u_encode(&mailbox);
    if msg["t"] == "unsubscribe" {
        subscribed.remove(&mailbox);
        if presence::remove_subscriber(st, conn, mailbox) {
            presence::announce(st, mailbox, false).await;
        }
        return Ok(json!({"t": "unsubscribed", "mailbox_id": id}));
    }
    if !subscribed.contains(&mailbox) && subscribed.len() >= presence::MAX_SUBSCRIPTIONS {
        return Err(DataError::BadRequest("too many subscriptions"));
    }
    // bez adresáře nemá kdo anchory spočítat
    data::plan(st, &mailbox)?;
    if !presence::add_subscriber(st, conn, tx.clone(), mailbox) {
        return Err(DataError::Storage);
    }
    subscribed.insert(mailbox);
    let anchors = presence::announce(st, mailbox, true).await;
    Ok(json!({"t": "subscribed", "mailbox_id": id, "anchors": anchors}))
}

/// Token bucket na jedno spojení: `rate` zpráv za sekundu, okamžitě až dvojnásobek.
struct Bucket {
    rate: f64,
    capacity: f64,
    tokens: f64,
    last: Instant,
}

impl Bucket {
    fn new(rate: u64) -> Self {
        let rate = rate.max(1) as f64;
        Self { rate, capacity: rate * 2.0, tokens: rate * 2.0, last: Instant::now() }
    }

    fn take(&mut self) -> bool {
        let now = Instant::now();
        self.tokens = (self.tokens + now.duration_since(self.last).as_secs_f64() * self.rate).min(self.capacity);
        self.last = now;
        if self.tokens < 1.0 {
            return false;
        }
        self.tokens -= 1.0;
        true
    }
}

fn attach_rid(reply: &mut Value, request: &Value) {
    if let Some(rid) = request["rid"].as_str().filter(|r| r.len() <= 64) {
        reply["rid"] = json!(rid);
    }
}

fn error_reply(e: &DataError, request: Option<&Value>) -> Value {
    let mut reply = json!({"t": "error", "code": e.code(), "reason": e.to_string()});
    if let Some(req) = request {
        attach_rid(&mut reply, req);
    }
    reply
}

/// Zpracuje `send`, `fetch` nebo `ack` (`spec/PROTOCOL.md` §13.3).
async fn handle(st: &Arc<NodeState>, msg: &Value) -> Value {
    let result = match msg["t"].as_str() {
        Some("send") => do_send(st, msg).await,
        Some("fetch") => do_fetch(st, msg).await,
        Some("ack") => do_ack(st, msg).await,
        _ => Err(DataError::BadRequest("unknown message type")),
    };
    let mut reply = result.unwrap_or_else(|e| error_reply(&e, None));
    attach_rid(&mut reply, msg);
    reply
}

fn field<'a>(msg: &'a Value, name: &'static str) -> Result<&'a str, DataError> {
    msg[name].as_str().ok_or(DataError::BadRequest(name))
}

async fn do_send(st: &Arc<NodeState>, msg: &Value) -> Result<Value, DataError> {
    let mailbox = data::parse_mailbox(field(msg, "mailbox_id")?)?;
    let id = data::parse_id(field(msg, "id")?)?;
    let ct = data::parse_ct(field(msg, "ct")?)?;
    let ts = msg["ts"].as_u64().ok_or(DataError::BadRequest("ts"))?;
    data::check_ts(ts, unix_now())?;
    let push_id = wake::optional_push_id(&msg["push_id"])?;
    let acks = data::send_with_push(st, mailbox, id, ts, ct, push_id).await?;
    Ok(json!({"t": "sent", "id": b64u_encode(&id), "acks": acks}))
}

async fn do_fetch(st: &Arc<NodeState>, msg: &Value) -> Result<Value, DataError> {
    let mailbox = data::parse_mailbox(field(msg, "mailbox_id")?)?;
    let list = data::fetch(st, mailbox, data::clamp_limit(msg["limit"].as_u64())).await?;
    Ok(json!({"t": "messages", "mailbox_id": b64u_encode(&mailbox), "messages": list.iter().map(|m| m.to_json(false)).collect::<Vec<_>>()}))
}

async fn do_ack(st: &Arc<NodeState>, msg: &Value) -> Result<Value, DataError> {
    let mailbox = data::parse_mailbox(field(msg, "mailbox_id")?)?;
    let ids = peer::parse_ids(&msg["ids"])?;
    let n = ids.len();
    data::ack(st, mailbox, ids).await?;
    Ok(json!({"t": "acked", "n": n}))
}
