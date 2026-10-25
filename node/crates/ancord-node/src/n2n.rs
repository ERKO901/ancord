//! Strana volajícího zabezpečeného kanálu node↔node (`spec/PROTOCOL.md` §14.2). Responder je v `peer.rs`.

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{json, Value};

use crate::channel::{self, Ephemeral, SessionKeys};
use crate::encoding::{b64u_decode, b64u_decode_array, b64u_encode};
use crate::peer::SESSION_TTL_S;
use crate::sign::{self, auth_header, DOMAIN_N2N_RESP};
use crate::state::NodeState;
use crate::ticket::DirectoryNode;
use crate::time::unix_now;

/// Session se obnoví dřív, než ji responder zahodí.
const RENEW_MARGIN_S: u64 = 30;

pub struct OutSession {
    sid: [u8; 16],
    keys: SessionKeys,
    renew_at: u64,
}

pub type OutSessions = std::sync::Mutex<HashMap<String, Arc<tokio::sync::Mutex<Option<Arc<OutSession>>>>>>;

/// Výsledek navázání session: buď session, nebo stav, kterým responder handshake odmítl (např. `403`, když ještě nezná
/// naši epochu).
enum Established {
    Session(Arc<OutSession>),
    Refused(u16),
}

enum Exchange {
    Done(u16, Value),
    UnknownSession,
}

fn url(st: &NodeState, node: &DirectoryNode, path: &str) -> String {
    format!("{}{path}", st.cfg.peer_url_template.replace("{host}", &node.host))
}

async fn handshake(st: &NodeState, node: &DirectoryNode) -> Result<Established, String> {
    let responder_pk = b64u_decode_array::<32>(&node.pk).ok_or("malformed node key in directory")?;
    let ce = Ephemeral::generate();
    let ce_pub = ce.public();
    let bytes = json!({"ce": b64u_encode(&ce_pub)}).to_string().into_bytes();
    let nonce = channel::random_array::<16>();
    let header = auth_header(st.identity.signing_key(), "POST", "/n/hs", &bytes, unix_now(), &nonce);
    let res = st
        .peer_http
        .post(url(st, node, "/n/hs"))
        .header("authorization", header)
        .header("content-type", "application/json")
        .body(bytes)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        return Ok(Established::Refused(res.status().as_u16()));
    }
    let answer: Value = res.json().await.map_err(|e| e.to_string())?;
    let (Some(ne), Some(sid), Some(sig)) = (
        answer["ne"].as_str().and_then(b64u_decode_array::<32>),
        answer["sid"].as_str().and_then(b64u_decode_array::<16>),
        answer["sig"].as_str().and_then(b64u_decode_array::<64>),
    ) else {
        return Err(format!("{} sent a malformed handshake", node.id));
    };
    let th = channel::n2n_transcript(&st.identity.pubkey(), &responder_pk, &nonce, &ce_pub, &ne, &sid);
    // odpověď musí podepsat identita, kterou nám potvrdil adresář (podepsaný Workerem), ne kdokoli na cestě
    if !sign::verify(&responder_pk, DOMAIN_N2N_RESP, &th, &sig) {
        return Err(format!("{} failed to prove its identity", node.id));
    }
    let shared = ce.diffie_hellman(&ne).map_err(|e| e.to_string())?;
    let renew_at = unix_now() + SESSION_TTL_S - RENEW_MARGIN_S;
    Ok(Established::Session(Arc::new(OutSession { sid, keys: SessionKeys::derive(&shared, &th), renew_at })))
}

async fn session_for(st: &NodeState, node: &DirectoryNode) -> Result<Established, String> {
    let slot = {
        let mut map = st.n2n_out.lock().map_err(|_| "session table poisoned")?;
        map.entry(node.id.clone()).or_default().clone()
    };
    // handshake se serializuje per node, ostatní nody tím nečekají
    let mut guard = slot.lock().await;
    if let Some(s) = guard.as_ref().filter(|s| s.renew_at > unix_now()) {
        return Ok(Established::Session(s.clone()));
    }
    match handshake(st, node).await? {
        Established::Session(s) => {
            *guard = Some(s.clone());
            Ok(Established::Session(s))
        }
        refused => {
            *guard = None;
            Ok(refused)
        }
    }
}

async fn forget(st: &NodeState, node: &DirectoryNode, sid: &[u8; 16]) {
    let slot = st.n2n_out.lock().ok().and_then(|m| m.get(&node.id).cloned());
    if let Some(slot) = slot {
        let mut guard = slot.lock().await;
        if guard.as_ref().is_some_and(|s| &s.sid == sid) {
            *guard = None;
        }
    }
}

async fn exchange(st: &NodeState, node: &DirectoryNode, session: &OutSession, op: &str, body: &Value) -> Result<Exchange, String> {
    let nonce = channel::random_array::<24>();
    let plain = json!({"op": op, "ts": unix_now(), "body": body}).to_string();
    let c = channel::seal_request(&session.keys.i2r, &session.sid, &nonce, plain.as_bytes()).map_err(|e| e.to_string())?;
    let envelope = json!({"sid": b64u_encode(&session.sid), "n": b64u_encode(&nonce), "c": b64u_encode(&c)});
    let res = st.peer_http.post(url(st, node, "/n/x")).json(&envelope).send().await.map_err(|e| e.to_string())?;
    let status = res.status().as_u16();
    if status != 200 {
        let outer: Value = res.json().await.unwrap_or(Value::Null);
        if status == 401 && outer["error"]["code"] == "unknown_session" {
            return Ok(Exchange::UnknownSession);
        }
        return Ok(Exchange::Done(status, outer));
    }
    let answer: Value = res.json().await.map_err(|e| e.to_string())?;
    let (Some(rn), Some(rc)) = (answer["n"].as_str().and_then(b64u_decode_array::<24>), answer["c"].as_str().and_then(b64u_decode)) else {
        return Err(format!("{} sent a malformed response", node.id));
    };
    let plain = channel::open_response(&session.keys.r2i, &session.sid, &nonce, &rn, &rc).map_err(|e| format!("{}: {e}", node.id))?;
    let inner: Value = serde_json::from_slice(&plain).map_err(|e| e.to_string())?;
    let status = inner["status"].as_u64().and_then(|s| u16::try_from(s).ok()).ok_or("malformed response status")?;
    Ok(Exchange::Done(status, inner["body"].clone()))
}

/// Zavolá operaci `op` (`put`, `get`, `ack`, `presence`, `deliver`) na jiném nodu. Vrací stav a tělo odpovědi.
/// Session se naváže nebo obnoví sama; chybějící session u responderu (restart) se zopakuje jednou.
pub async fn call(st: &NodeState, node: &DirectoryNode, op: &str, body: &Value) -> Result<(u16, Value), String> {
    for _ in 0..2 {
        let session = match session_for(st, node).await? {
            Established::Session(s) => s,
            Established::Refused(status) => return Ok((status, Value::Null)),
        };
        match exchange(st, node, &session, op, body).await? {
            Exchange::Done(status, answer) => return Ok((status, answer)),
            Exchange::UnknownSession => forget(st, node, &session.sid).await,
        }
    }
    Err(format!("{} keeps refusing the session", node.id))
}
