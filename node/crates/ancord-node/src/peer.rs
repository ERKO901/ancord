//! Node↔node API anchoru (`spec/PROTOCOL.md` §13.4) přes zabezpečený kanál (§14.2).
//!
//! Cloudflare Tunnel vidí HTTP provoz mezi nody, proto se nepoužívá holé JSON API. `POST /n/hs` naváže session
//! (podepsaný request, efemerní X25519, odpověď podepsaná identitou nodu) a `POST /n/x` nese šifrované operace
//! `put`, `get`, `ack`, `presence`, `deliver` a operace profilů, signálů a stavu doručení (§15.9). Žádná z operací není dostupná jinak.

use std::collections::HashMap;
use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::{header, HeaderMap, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use serde_json::{json, Value};

use crate::channel::{self, Ephemeral, SessionKeys};
use crate::data::{self, DataError, MAX_ACK_IDS};
use crate::encoding::{b64u_decode_array, b64u_encode};
use crate::presence;
use crate::profile::{self, Profile};
use crate::sign::{verify_request, DOMAIN_N2N_RESP};
use crate::state::NodeState;
use crate::store::PutOutcome;
use crate::time::unix_now;
use crate::wake;

const SKEW_S: u64 = 60;
const MAX_BODY: usize = 256 * 1024;
/// Jak dlouho žije session; po vypršení se naváže nová (forward secrecy a rotace klíčů).
pub const SESSION_TTL_S: u64 = 600;
/// Strop počtu session, které node drží jako responder.
const MAX_SESSIONS: usize = 4096;
/// Strop počtu session jednoho volajícího.
const MAX_SESSIONS_PER_PEER: usize = 8;

/// Session, kterou node drží jako responder.
pub struct InSession {
    peer: [u8; 32],
    keys: SessionKeys,
    exp: u64,
}

pub type InSessions = std::sync::Mutex<HashMap<[u8; 16], Arc<InSession>>>;

/// `GET /n/directory`: podepsaná obálka adresáře, který node drží (spec §6.1). Veřejná data, podpis ověří příjemce.
async fn directory(State(st): State<Arc<NodeState>>) -> Response {
    match st.directory_envelope() {
        Some(env) => Json((*env).clone()).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub fn routes() -> Router<Arc<NodeState>> {
    Router::new()
        .route("/n/hs", post(handshake))
        .route("/n/x", post(exchange))
        .route("/n/directory", axum::routing::get(directory))
        .layer(DefaultBodyLimit::max(MAX_BODY))
}

/// Chyba operace; do odpovědi jde zašifrovaná.
#[derive(Debug)]
pub struct ApiError {
    pub status: u16,
    pub code: &'static str,
    pub message: String,
    /// Obtížnost razítka u `pow_required` (§15.13).
    pub bits: Option<u8>,
}

impl ApiError {
    fn new(status: u16, code: &'static str, message: impl Into<String>) -> Self {
        Self { status, code, message: message.into(), bits: None }
    }

    fn body(&self) -> Value {
        let mut e = json!({"code": self.code, "message": self.message});
        if let Some(bits) = self.bits {
            e["bits"] = json!(bits);
        }
        json!({"error": e})
    }
}

impl From<ancord_core::error::ProofError> for ApiError {
    fn from(e: ancord_core::error::ProofError) -> Self {
        Self::from(DataError::from(e))
    }
}

impl From<DataError> for ApiError {
    fn from(e: DataError) -> Self {
        match e {
            DataError::MailboxFull => Self::new(507, "mailbox_full", e.to_string()),
            DataError::BadRequest(_) => Self::new(400, "bad_request", e.to_string()),
            DataError::TooLarge => Self::new(413, "too_large", e.to_string()),
            DataError::InvalidProof => Self::new(400, "invalid_proof", e.to_string()),
            DataError::NotFound => Self::new(404, "not_found", e.to_string()),
            DataError::PowRequired(bits) => Self { bits: Some(bits), ..Self::new(428, "pow_required", e.to_string()) },
            _ => Self::new(503, "unavailable", e.to_string()),
        }
    }
}

/// Chyba vnějšího HTTP (mimo šifrovanou vrstvu): nic tajného neobsahuje.
fn outer(status: StatusCode, code: &str, message: &str) -> Response {
    (status, Json(json!({"error": {"code": code, "message": message}}))).into_response()
}

// --- handshake -------------------------------------------------------------------------------------------------

/// `POST /n/hs`: podepsaný request volajícího (člen adresáře, bez replaye) s `{"ce"}`.
async fn handshake(State(st): State<Arc<NodeState>>, uri: Uri, headers: HeaderMap, bytes: Bytes) -> Response {
    let now = unix_now();
    let path = uri.path_and_query().map_or("/", |p| p.as_str());
    let auth = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|h| verify_request(h, "POST", path, &bytes, now, SKEW_S));
    let Some(auth) = auth else { return outer(StatusCode::UNAUTHORIZED, "unauthorized", "invalid signature") };
    if !data::is_member(&st, &auth.pubkey) {
        return outer(StatusCode::FORBIDDEN, "forbidden", "caller is not in the directory");
    }
    let key = format!("{}.{}", b64u_encode(&auth.pubkey), auth.nonce_b64);
    if !st.consume_nonce(key, now + 2 * SKEW_S + 5, now) {
        return outer(StatusCode::CONFLICT, "conflict", "request replayed");
    }
    let parsed = serde_json::from_slice::<Value>(&bytes).ok().and_then(|b| b["ce"].as_str().and_then(b64u_decode_array::<32>));
    let (Some(ce), Some(nonce)) = (parsed, b64u_decode_array::<16>(&auth.nonce_b64)) else {
        return outer(StatusCode::BAD_REQUEST, "bad_request", "invalid handshake");
    };

    let ne = Ephemeral::generate();
    let ne_pub = ne.public();
    let sid = channel::random_array::<16>();
    let me = st.identity.pubkey();
    let th = channel::n2n_transcript(&auth.pubkey, &me, &nonce, &ce, &ne_pub, &sid);
    let Ok(shared) = ne.diffie_hellman(&ce) else {
        return outer(StatusCode::BAD_REQUEST, "bad_request", "invalid key");
    };
    let session = Arc::new(InSession { peer: auth.pubkey, keys: SessionKeys::derive(&shared, &th), exp: now + SESSION_TTL_S });
    if !store_session(&st, sid, session, now) {
        return outer(StatusCode::SERVICE_UNAVAILABLE, "unavailable", "too many sessions");
    }
    let sig = st.identity.sign(DOMAIN_N2N_RESP, &th);
    Json(json!({"ne": b64u_encode(&ne_pub), "sid": b64u_encode(&sid), "sig": b64u_encode(&sig)})).into_response()
}

fn store_session(st: &NodeState, sid: [u8; 16], session: Arc<InSession>, now: u64) -> bool {
    let Ok(mut table) = st.n2n_in.lock() else { return false };
    table.retain(|_, s| s.exp > now);
    let mine = table.values().filter(|s| s.peer == session.peer).count();
    if table.len() >= MAX_SESSIONS || mine >= MAX_SESSIONS_PER_PEER {
        // místo staré session téhož volajícího uvolníme nejstarší, ať se nedá vyčerpat tabulka
        let oldest = table.iter().filter(|(_, s)| s.peer == session.peer).min_by_key(|(_, s)| s.exp).map(|(k, _)| *k);
        match oldest {
            Some(k) if mine >= MAX_SESSIONS_PER_PEER => {
                table.remove(&k);
            }
            _ => return false,
        }
    }
    table.insert(sid, session);
    true
}

// --- šifrované operace -----------------------------------------------------------------------------------------

/// `POST /n/x`: `{"sid","n","c"}`; uvnitř `{"op","ts","body"}`.
async fn exchange(State(st): State<Arc<NodeState>>, bytes: Bytes) -> Response {
    let now = unix_now();
    let envelope = serde_json::from_slice::<Value>(&bytes).ok();
    let parsed = envelope.as_ref().and_then(|e| {
        Some((
            e["sid"].as_str().and_then(b64u_decode_array::<16>)?,
            e["n"].as_str().and_then(b64u_decode_array::<24>)?,
            e["c"].as_str().and_then(crate::encoding::b64u_decode)?,
        ))
    });
    let Some((sid, nonce, ct)) = parsed else { return outer(StatusCode::BAD_REQUEST, "bad_request", "invalid envelope") };
    let session = st.n2n_in.lock().ok().and_then(|t| t.get(&sid).cloned()).filter(|s| s.exp > now);
    let Some(session) = session else {
        return outer(StatusCode::UNAUTHORIZED, "unknown_session", "session expired or unknown");
    };
    let Ok(plain) = channel::open_request(&session.keys.i2r, &sid, &nonce, &ct) else {
        return outer(StatusCode::UNAUTHORIZED, "unauthorized", "cannot decrypt");
    };
    // odesílatel musí být pořád v adresáři (jinak ho Worker mezitím vyřadil)
    if !data::is_member(&st, &session.peer) {
        if let Ok(mut t) = st.n2n_in.lock() {
            t.remove(&sid);
        }
        return outer(StatusCode::FORBIDDEN, "forbidden", "caller is not in the directory");
    }
    let request = serde_json::from_slice::<Value>(&plain).ok();
    let (status, body) = match request.as_ref().map(|r| (r["op"].as_str(), r["ts"].as_u64(), &r["body"])) {
        Some((Some(op), Some(ts), body)) if now.abs_diff(ts) <= SKEW_S => {
            let key = format!("{}.{}", b64u_encode(&sid), b64u_encode(&nonce));
            if st.consume_nonce(key, now + 2 * SKEW_S + 5, now) {
                match dispatch(&st, &session.peer, op, body).await {
                    Ok(v) => (200, v),
                    Err(e) => (e.status, e.body()),
                }
            } else {
                (409, json!({"error": {"code": "conflict", "message": "request replayed"}}))
            }
        }
        _ => (400, json!({"error": {"code": "bad_request", "message": "invalid request"}})),
    };
    let reply = json!({"status": status, "body": body}).to_string();
    let rn = channel::random_array::<24>();
    match channel::seal_response(&session.keys.r2i, &sid, &nonce, &rn, reply.as_bytes()) {
        Ok(c) => Json(json!({"n": b64u_encode(&rn), "c": b64u_encode(&c)})).into_response(),
        Err(_) => outer(StatusCode::INTERNAL_SERVER_ERROR, "internal", "cannot seal response"),
    }
}

async fn dispatch(st: &Arc<NodeState>, caller: &[u8; 32], op: &str, body: &Value) -> Result<Value, ApiError> {
    match op {
        "put" => put(st, body).await,
        "get" => get(st, body).await,
        "ack" => ack(st, body).await,
        "presence" => presence_update(st, caller, body),
        "deliver" => deliver(st, caller, body),
        "profile_put" => profile_put(st, body).await,
        "profile_get" => profile_get(st, body).await,
        "guard" => guard_put(st, body).await,
        "signal" => signal(st, body),
        "typing" => typing(st, caller, body),
        "status" => status(st, body).await,
        _ => Err(ApiError::new(400, "bad_request", "unknown operation")),
    }
}

/// Mailbox z těla; příjemce musí být jeho anchorem (aktuální nebo předchozí epocha).
fn anchored_mailbox(st: &NodeState, body: &Value) -> Result<[u8; 32], ApiError> {
    let mailbox = body["mailbox_id"].as_str().ok_or(DataError::BadRequest("missing mailbox_id")).and_then(data::parse_mailbox)?;
    if !data::is_anchor(st, &mailbox) {
        return Err(ApiError::new(403, "forbidden", "not an anchor of this mailbox"));
    }
    Ok(mailbox)
}

async fn put(st: &Arc<NodeState>, body: &Value) -> Result<Value, ApiError> {
    let mailbox = anchored_mailbox(st, body)?;
    let delivered = body["delivered"].as_bool().unwrap_or(false);
    let id = data::parse_id(body["id"].as_str().ok_or(DataError::BadRequest("missing id"))?)?;
    let ts = body["ts"].as_u64().ok_or(DataError::BadRequest("missing ts"))?;
    let exp = body["exp"].as_u64().ok_or(DataError::BadRequest("missing exp"))?;
    // náhrobek ct nemá
    let ct = if delivered { None } else { Some(data::parse_ct(body["ct"].as_str().ok_or(DataError::BadRequest("missing ct"))?)?) };
    let push_id = wake::optional_push_id(&body["push_id"])?;
    let ct_copy = ct.clone();
    let outcome = data::put_local(st, mailbox, id, ts, exp, ct).await?;
    if let (PutOutcome::Stored, Some(ct)) = (outcome, ct_copy) {
        presence::after_store(st, mailbox, id, ts, ct, push_id);
    }
    Ok(json!({"status": if outcome == PutOutcome::Stored { "stored" } else { "duplicate" }}))
}

/// Stráž schránky (§15.13): příjemce musí být anchorem `inbox_id(pk)` a stráž sám ověří.
async fn guard_put(st: &Arc<NodeState>, body: &Value) -> Result<Value, ApiError> {
    let guard = crate::guard::Guard::from_json(body).ok_or(DataError::BadRequest("invalid guard"))?;
    if !data::is_anchor(st, &guard.inbox()) {
        return Err(ApiError::new(403, "forbidden", "not an anchor of this inbox"));
    }
    let outcome = crate::guard::put_local(st, &guard).await?;
    Ok(json!({"status": if outcome == PutOutcome::Stored { "stored" } else { "duplicate" }}))
}

async fn get(st: &Arc<NodeState>, body: &Value) -> Result<Value, ApiError> {
    let mailbox = anchored_mailbox(st, body)?;
    let list = data::get_local(st, mailbox, data::clamp_limit(body["limit"].as_u64())).await?;
    Ok(json!({"messages": list.iter().map(|m| m.to_json(true)).collect::<Vec<_>>()}))
}

async fn ack(st: &Arc<NodeState>, body: &Value) -> Result<Value, ApiError> {
    let mailbox = anchored_mailbox(st, body)?;
    let ids = parse_ids(&body["ids"])?;
    Ok(json!({"n": data::ack_local(st, mailbox, ids).await?}))
}

/// Volající (gateway) oznamuje anchoru, že je mailbox online u něj. `exp = 0` presence ruší.
fn presence_update(st: &NodeState, caller: &[u8; 32], body: &Value) -> Result<Value, ApiError> {
    let mailbox = anchored_mailbox(st, body)?;
    let (Some(exp), Some(gateway)) = (body["exp"].as_u64(), data::node_by_pubkey(st, caller)) else {
        return Err(ApiError::new(400, "bad_request", "missing exp"));
    };
    presence::set_presence(st, mailbox, &gateway.id, exp);
    Ok(json!({"ok": true}))
}

/// Anchor předává novou zprávu gateway, u které je příjemce online.
fn deliver(st: &NodeState, caller: &[u8; 32], body: &Value) -> Result<Value, ApiError> {
    let mailbox = data::parse_mailbox(body["mailbox_id"].as_str().ok_or(DataError::BadRequest("missing mailbox_id"))?)?;
    let id = data::parse_id(body["id"].as_str().ok_or(DataError::BadRequest("missing id"))?)?;
    let ts = body["ts"].as_u64().ok_or(DataError::BadRequest("missing ts"))?;
    let ct = data::parse_ct(body["ct"].as_str().ok_or(DataError::BadRequest("missing ct"))?)?;
    if !data::is_anchor_pubkey(st, caller, &mailbox) {
        return Err(ApiError::new(403, "forbidden", "caller is not an anchor of this mailbox"));
    }
    Ok(json!({"delivered": presence::deliver_local(st, &mailbox, id, ts, &ct)}))
}

/// Profil uživatele od jiného nodu. Příjemce ho ověří znovu a musí být anchorem `profile_id`.
async fn profile_put(st: &Arc<NodeState>, body: &Value) -> Result<Value, ApiError> {
    let profile = Profile::from_json(body).ok_or(DataError::BadRequest("invalid profile"))?;
    if !data::is_anchor(st, &profile::profile_id(&profile.pk)) {
        return Err(ApiError::new(403, "forbidden", "not an anchor of this profile"));
    }
    let outcome = profile::put_local(st, &profile).await?;
    Ok(json!({"status": if outcome == PutOutcome::Stored { "stored" } else { "duplicate" }}))
}

async fn profile_get(st: &Arc<NodeState>, body: &Value) -> Result<Value, ApiError> {
    let pk = profile::parse_pk(body["pk"].as_str().ok_or(DataError::BadRequest("missing pk"))?)?;
    if !data::is_anchor(st, &profile::profile_id(&pk)) {
        return Err(ApiError::new(403, "forbidden", "not an anchor of this profile"));
    }
    let found = profile::get_local(st, pk).await?;
    Ok(json!({"profile": found.map(|p| p.to_json())}))
}

/// Gateway odesílatele hlásí anchoru, že odesílatel píše. Předání gateway příjemce běží na pozadí.
fn signal(st: &Arc<NodeState>, body: &Value) -> Result<Value, ApiError> {
    let mailbox = anchored_mailbox(st, body)?;
    let on = body["on"].as_bool().ok_or(DataError::BadRequest("missing on"))?;
    let st = st.clone();
    tokio::spawn(async move { presence::relay_typing(&st, &mailbox, on).await });
    Ok(json!({"ok": true}))
}

/// Anchor předává signál psaní gateway příjemce.
fn typing(st: &NodeState, caller: &[u8; 32], body: &Value) -> Result<Value, ApiError> {
    let mailbox = data::parse_mailbox(body["mailbox_id"].as_str().ok_or(DataError::BadRequest("missing mailbox_id"))?)?;
    let on = body["on"].as_bool().ok_or(DataError::BadRequest("missing on"))?;
    if !data::is_anchor_pubkey(st, caller, &mailbox) {
        return Err(ApiError::new(403, "forbidden", "caller is not an anchor of this mailbox"));
    }
    Ok(json!({"delivered": presence::deliver_typing(st, &mailbox, on)}))
}

async fn status(st: &Arc<NodeState>, body: &Value) -> Result<Value, ApiError> {
    let mailbox = anchored_mailbox(st, body)?;
    let ids = parse_ids(&body["ids"])?;
    let states = data::status_local(st, mailbox, ids).await?;
    Ok(json!({"states": states.iter().map(|s| s.as_str()).collect::<Vec<_>>()}))
}

/// Seznam `id` zpráv (`ack`), nejvýše `MAX_ACK_IDS`.
pub fn parse_ids(v: &Value) -> Result<Vec<[u8; 16]>, DataError> {
    let list = v.as_array().ok_or(DataError::BadRequest("ids must be an array"))?;
    if list.len() > MAX_ACK_IDS {
        return Err(DataError::BadRequest("too many ids"));
    }
    list.iter().map(|i| i.as_str().ok_or(DataError::BadRequest("invalid id")).and_then(data::parse_id)).collect()
}

