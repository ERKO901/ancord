//! HTTP server nodu (cíl ingressu tunelu): důkaz hostname pro Worker a zdroj dat pro peer witness.
//! Ve stejném routeru je admission klientů (`/client`) a node↔node API anchoru (`/n/*`).

use std::collections::HashMap;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::{header, HeaderMap, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use rand::RngCore;
use serde_json::json;

use crate::encoding::{b64u_decode_array, b64u_encode};
use crate::sign::{verify_request, DOMAIN_PROOF};
use crate::state::NodeState;
use crate::time::unix_now;

pub const PROOF_PATH: &str = "/.well-known/melo-proof";
pub const BW_PATH: &str = "/.well-known/melo-bw";
const SKEW_S: u64 = 60;
const MAX_WITNESS_BYTES: u64 = 64 * 1024 * 1024;

pub fn router(state: Arc<NodeState>) -> Router {
    Router::new()
        .route(PROOF_PATH, get(proof))
        .route(BW_PATH, get(bandwidth_source))
        .route("/client", get(crate::client::upgrade))
        .merge(crate::peer::routes())
        .route("/health", get(|| async { Json(json!({"ok": true})) }))
        .with_state(state)
}

/// `GET /.well-known/melo-proof?c=<b64u 32 B>` → `{pubkey, signature}` (spec §4.3).
async fn proof(State(st): State<Arc<NodeState>>, Query(q): Query<HashMap<String, String>>) -> Response {
    let Some(challenge) = q.get("c").and_then(|c| b64u_decode_array::<32>(c)) else {
        return (StatusCode::BAD_REQUEST, "bad challenge").into_response();
    };
    let hostname = match st.hostname() {
        Some(h) => h,
        None => return (StatusCode::SERVICE_UNAVAILABLE, "not registered yet").into_response(),
    };
    let mut payload = challenge.to_vec();
    payload.extend_from_slice(hostname.as_bytes());
    payload.extend_from_slice(&st.identity.pubkey());
    let sig = st.identity.sign(DOMAIN_PROOF, &payload);
    Json(json!({"pubkey": st.identity.pubkey_b64(), "signature": b64u_encode(&sig)})).into_response()
}

/// Zdroj dat pro peer witness (spec §9.4): jen pro podepsané requesty nodů z aktuálního adresáře.
async fn bandwidth_source(
    State(st): State<Arc<NodeState>>,
    uri: Uri,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let now = unix_now();
    let Some(bytes) = q.get("bytes").and_then(|b| b.parse::<u64>().ok()).filter(|b| (1..=MAX_WITNESS_BYTES).contains(b)) else {
        return (StatusCode::BAD_REQUEST, "bad bytes").into_response();
    };
    let path = uri.path_and_query().map_or(BW_PATH, |p| p.as_str());
    let auth = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|h| verify_request(h, "GET", path, &[], now, SKEW_S));
    let Some(auth) = auth else {
        return (StatusCode::UNAUTHORIZED, "invalid signature").into_response();
    };
    let known = st.directory().is_some_and(|d| {
        d.nodes.iter().any(|n| b64u_decode_array::<32>(&n.pk).is_some_and(|pk| pk == auth.pubkey))
    });
    if !known {
        return (StatusCode::FORBIDDEN, "unknown node").into_response();
    }
    let key = format!("{}.{}", b64u_encode(&auth.pubkey), auth.nonce_b64);
    if !st.consume_nonce(key, now + 2 * SKEW_S + 5, now) {
        return (StatusCode::CONFLICT, "request replayed").into_response();
    }
    let stream = futures_util::stream::unfold(bytes, |left| async move {
        if left == 0 {
            return None;
        }
        let n = left.min(64 * 1024) as usize;
        let mut chunk = vec![0u8; n];
        rand::rngs::OsRng.fill_bytes(&mut chunk);
        Some((Ok::<_, std::convert::Infallible>(chunk), left - n as u64))
    });
    Response::builder()
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_LENGTH, bytes)
        .header(header::CACHE_CONTROL, "no-store")
        .body(Body::from_stream(stream))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}
