//! Node↔node API anchoru: `/n/put`, `/n/get`, `/n/ack` (`spec/PROTOCOL.md` §13.4).
//! Každý request je podepsaný identitou volajícího nodu (§3) a volající musí být v adresáři.

use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::{header, HeaderMap, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use serde_json::{json, Value};

use crate::data::{self, DataError, MAX_ACK_IDS};
use crate::encoding::b64u_encode;
use crate::sign::verify_request;
use crate::state::NodeState;
use crate::store::PutOutcome;
use crate::time::unix_now;
use crate::wake;

const SKEW_S: u64 = 60;
const MAX_BODY: usize = 128 * 1024;

pub fn routes() -> Router<Arc<NodeState>> {
    Router::new()
        .route("/n/put", post(put))
        .route("/n/get", post(get))
        .route("/n/ack", post(ack))
        .layer(DefaultBodyLimit::max(MAX_BODY))
}

fn error(status: StatusCode, code: &str, message: &str) -> Response {
    (status, Json(json!({"error": {"code": code, "message": message}}))).into_response()
}

fn data_error(e: &DataError) -> Response {
    match e {
        DataError::MailboxFull => error(StatusCode::INSUFFICIENT_STORAGE, "mailbox_full", &e.to_string()),
        DataError::BadRequest(_) => error(StatusCode::BAD_REQUEST, "bad_request", &e.to_string()),
        DataError::TooLarge => error(StatusCode::PAYLOAD_TOO_LARGE, "too_large", &e.to_string()),
        _ => error(StatusCode::SERVICE_UNAVAILABLE, "unavailable", &e.to_string()),
    }
}

/// Ověří podpis, členství volajícího v adresáři a replay; vrací tělo jako JSON.
fn authenticate(st: &NodeState, uri: &Uri, headers: &HeaderMap, body: &[u8]) -> Result<Value, Box<Response>> {
    let now = unix_now();
    let path = uri.path_and_query().map_or("/", |p| p.as_str());
    let auth = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|h| verify_request(h, "POST", path, body, now, SKEW_S))
        .ok_or_else(|| Box::new(error(StatusCode::UNAUTHORIZED, "unauthorized", "invalid signature")))?;
    if !data::is_member(st, &auth.pubkey) {
        return Err(Box::new(error(StatusCode::FORBIDDEN, "forbidden", "caller is not in the directory")));
    }
    let key = format!("{}.{}", b64u_encode(&auth.pubkey), auth.nonce_b64);
    if !st.consume_nonce(key, now + 2 * SKEW_S + 5, now) {
        return Err(Box::new(error(StatusCode::CONFLICT, "conflict", "request replayed")));
    }
    serde_json::from_slice(body).map_err(|_| Box::new(error(StatusCode::BAD_REQUEST, "bad_request", "invalid JSON")))
}

/// Mailbox z těla; příjemce musí být jeho anchorem (aktuální nebo předchozí epocha).
fn anchored_mailbox(st: &NodeState, body: &Value) -> Result<[u8; 32], Box<Response>> {
    let mailbox = body["mailbox_id"].as_str().ok_or(DataError::BadRequest("missing mailbox_id")).and_then(data::parse_mailbox).map_err(|e| Box::new(data_error(&e)))?;
    if !data::is_anchor(st, &mailbox) {
        return Err(Box::new(error(StatusCode::FORBIDDEN, "forbidden", "not an anchor of this mailbox")));
    }
    Ok(mailbox)
}

async fn put(State(st): State<Arc<NodeState>>, uri: Uri, headers: HeaderMap, bytes: Bytes) -> Response {
    let body = match authenticate(&st, &uri, &headers, &bytes) {
        Ok(b) => b,
        Err(r) => return *r,
    };
    let mailbox = match anchored_mailbox(&st, &body) {
        Ok(m) => m,
        Err(r) => return *r,
    };
    let delivered = body["delivered"].as_bool().unwrap_or(false);
    let parsed = (|| {
        let id = data::parse_id(body["id"].as_str().ok_or(DataError::BadRequest("missing id"))?)?;
        let ts = body["ts"].as_u64().ok_or(DataError::BadRequest("missing ts"))?;
        let exp = body["exp"].as_u64().ok_or(DataError::BadRequest("missing exp"))?;
        // náhrobek ct nemá
        let ct = if delivered { None } else { Some(data::parse_ct(body["ct"].as_str().ok_or(DataError::BadRequest("missing ct"))?)?) };
        Ok::<_, DataError>((id, ts, exp, ct))
    })();
    let (id, ts, exp, ct) = match parsed {
        Ok(p) => p,
        Err(e) => return data_error(&e),
    };
    let push_id = match wake::optional_push_id(&body["push_id"]) {
        Ok(p) => p,
        Err(e) => return data_error(&e),
    };
    let has_ct = ct.is_some();
    match data::put_local(&st, mailbox, id, ts, exp, ct).await {
        Ok(outcome) => {
            if let (PutOutcome::Stored, true, Some(push_id)) = (outcome, has_ct, push_id) {
                wake::spawn(&st, push_id, mailbox);
            }
            let status = if outcome == PutOutcome::Stored { "stored" } else { "duplicate" };
            Json(json!({"status": status})).into_response()
        }
        Err(e) => data_error(&e),
    }
}

async fn get(State(st): State<Arc<NodeState>>, uri: Uri, headers: HeaderMap, bytes: Bytes) -> Response {
    let body = match authenticate(&st, &uri, &headers, &bytes) {
        Ok(b) => b,
        Err(r) => return *r,
    };
    let mailbox = match anchored_mailbox(&st, &body) {
        Ok(m) => m,
        Err(r) => return *r,
    };
    match data::get_local(&st, mailbox, data::clamp_limit(body["limit"].as_u64())).await {
        Ok(list) => Json(json!({"messages": list.iter().map(|m| m.to_json(true)).collect::<Vec<_>>()})).into_response(),
        Err(e) => data_error(&e),
    }
}

async fn ack(State(st): State<Arc<NodeState>>, uri: Uri, headers: HeaderMap, bytes: Bytes) -> Response {
    let body = match authenticate(&st, &uri, &headers, &bytes) {
        Ok(b) => b,
        Err(r) => return *r,
    };
    let mailbox = match anchored_mailbox(&st, &body) {
        Ok(m) => m,
        Err(r) => return *r,
    };
    let ids = match parse_ids(&body["ids"]) {
        Ok(ids) => ids,
        Err(e) => return data_error(&e),
    };
    match data::ack_local(&st, mailbox, ids).await {
        Ok(n) => Json(json!({"n": n})).into_response(),
        Err(e) => data_error(&e),
    }
}

/// Seznam `id` zpráv (`ack`), nejvýše `MAX_ACK_IDS`.
pub fn parse_ids(v: &Value) -> Result<Vec<[u8; 16]>, DataError> {
    let list = v.as_array().ok_or(DataError::BadRequest("ids must be an array"))?;
    if list.len() > MAX_ACK_IDS {
        return Err(DataError::BadRequest("too many ids"));
    }
    list.iter().map(|i| i.as_str().ok_or(DataError::BadRequest("invalid id")).and_then(data::parse_id)).collect()
}
