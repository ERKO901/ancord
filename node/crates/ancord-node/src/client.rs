//! Admission klientů: WebSocket `/client` s tiketem od Workeru (`spec/PROTOCOL.md` §8.1).

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use rand::RngCore;
use serde_json::{json, Value};

use crate::data::{self, DataError};
use crate::encoding::{b64u_decode_array, b64u_encode};
use crate::peer;
use crate::sign::{self, DOMAIN_CLIENT_AUTH};
use crate::state::NodeState;
use crate::ticket::{verify_ticket, TicketBody, TicketError};
use crate::time::unix_now;
use crate::wake;

const MAX_MESSAGE_BYTES: usize = 64 * 1024;
pub const CLOSE_UNAUTHORIZED: u16 = 4401;
pub const CLOSE_AUTH_TIMEOUT: u16 = 4408;
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

async fn close(socket: &mut WebSocket, code: u16, reason: &str) {
    let _ = socket.send(Message::Close(Some(CloseFrame { code, reason: reason.to_owned().into() }))).await;
}

async fn reject(socket: &mut WebSocket, reason: &str) {
    let _ = socket.send(Message::text(json!({"t": "error", "reason": reason}).to_string())).await;
    close(socket, CLOSE_UNAUTHORIZED, reason).await;
}

/// Ověří `auth`: ticket, podpis klíčem `ek` přes výzvu, jednorázovost. Ticket se spotřebuje až po úspěšném podpisu,
/// aby ho nemohl vypálit někdo, kdo má ticket, ale ne klíč.
fn authenticate(st: &NodeState, challenge: &[u8; 32], msg: &Value) -> Result<TicketBody, &'static str> {
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
    let mut payload = challenge.to_vec();
    payload.extend_from_slice(st.identity.node_id().as_bytes());
    if !sign::verify(&ek, DOMAIN_CLIENT_AUTH, &payload, &sig) {
        return Err("invalid client signature");
    }
    st.tickets.consume(&body, now).map_err(|_| "ticket already used")?;
    Ok(body)
}

async fn session(st: Arc<NodeState>, mut socket: WebSocket) {
    if st.conns.load(Ordering::Relaxed) >= st.cfg.max_conns {
        close(&mut socket, CLOSE_OVERLOADED, "node is full").await;
        return;
    }
    let _guard = ConnGuard::new(&st);

    let mut challenge = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut challenge);
    let hello = json!({"t": "hello", "node_id": st.identity.node_id(), "challenge": b64u_encode(&challenge)});
    if socket.send(Message::text(hello.to_string())).await.is_err() {
        return;
    }

    let timeout = Duration::from_secs(st.cfg.auth_timeout_s);
    let first = match tokio::time::timeout(timeout, socket.recv()).await {
        Err(_) => return close(&mut socket, CLOSE_AUTH_TIMEOUT, "auth timeout").await,
        Ok(None | Some(Err(_))) => return,
        Ok(Some(Ok(m))) => m,
    };
    let Message::Text(text) = first else {
        return reject(&mut socket, "expected text frame").await;
    };
    let Ok(msg) = serde_json::from_str::<Value>(text.as_str()) else {
        return reject(&mut socket, "invalid JSON").await;
    };
    if let Err(reason) = authenticate(&st, &challenge, &msg) {
        return reject(&mut socket, reason).await;
    }
    if socket.send(Message::text(json!({"t": "ready"}).to_string())).await.is_err() {
        return;
    }

    let mut bucket = Bucket::new(st.cfg.client_msg_rate);
    while let Some(Ok(frame)) = socket.recv().await {
        match frame {
            Message::Text(text) => {
                let Ok(msg) = serde_json::from_str::<Value>(text.as_str()) else {
                    let reply = error_reply(&DataError::BadRequest("invalid JSON"), None);
                    if socket.send(Message::text(reply.to_string())).await.is_err() {
                        return;
                    }
                    continue;
                };
                let reply = if msg["t"] == "ping" {
                    json!({"t": "pong"})
                } else if !bucket.take() {
                    let mut r = json!({"t": "error", "code": "rate_limited", "reason": "too many messages"});
                    attach_rid(&mut r, &msg);
                    r
                } else {
                    handle(&st, &msg).await
                };
                if socket.send(Message::text(reply.to_string())).await.is_err() {
                    return;
                }
            }
            Message::Close(_) => return,
            _ => {}
        }
    }
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
