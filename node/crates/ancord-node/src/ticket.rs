//! Ověření ticketu a podepsaného adresáře Workeru (`spec/PROTOCOL.md` §6 a §8).

use std::collections::HashMap;
use std::sync::Mutex;

use serde::Deserialize;
use serde_json::Value;

use crate::canonical::canonical_bytes;
use crate::encoding::{b64u_decode, b64u_decode_array};
use crate::sign::{self, DOMAIN_DIRECTORY, DOMAIN_TICKET};

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct TicketBody {
    pub v: u64,
    pub nid: String,
    pub ek: String,
    pub exp: u64,
    pub n: String,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum TicketError {
    #[error("malformed ticket")]
    Malformed,
    #[error("bad ticket signature")]
    BadSignature,
    #[error("ticket not issued for this node")]
    WrongNode,
    #[error("ticket expired")]
    Expired,
    #[error("ticket already used")]
    Replayed,
}

/// Ověří podpis, kanonickou podobu, `nid` a platnost. Jednorázovost řeší `TicketGuard`.
pub fn verify_ticket(worker_key: &[u8; 32], ticket: &str, node_id: &str, now: u64) -> Result<TicketBody, TicketError> {
    let (payload_b64, sig_b64) = ticket.split_once('.').ok_or(TicketError::Malformed)?;
    if sig_b64.contains('.') {
        return Err(TicketError::Malformed);
    }
    let payload = b64u_decode(payload_b64).ok_or(TicketError::Malformed)?;
    let sig: [u8; 64] = b64u_decode_array(sig_b64).ok_or(TicketError::Malformed)?;
    if !sign::verify(worker_key, DOMAIN_TICKET, &payload, &sig) {
        return Err(TicketError::BadSignature);
    }
    let value: Value = serde_json::from_slice(&payload).map_err(|_| TicketError::Malformed)?;
    // musí jít přesně o kanonické bajty, jinak by jeden ticket měl víc podob
    if canonical_bytes(&value).map_err(|_| TicketError::Malformed)? != payload {
        return Err(TicketError::Malformed);
    }
    let body: TicketBody = serde_json::from_value(value).map_err(|_| TicketError::Malformed)?;
    if body.v != 1 || b64u_decode_array::<32>(&body.ek).is_none() {
        return Err(TicketError::Malformed);
    }
    if body.nid != node_id {
        return Err(TicketError::WrongNode);
    }
    if body.exp <= now {
        return Err(TicketError::Expired);
    }
    Ok(body)
}

/// Pamatuje použitá `n` do vypršení ticketu (admission je jednorázový).
#[derive(Default)]
pub struct TicketGuard {
    seen: Mutex<HashMap<String, u64>>,
}

impl TicketGuard {
    pub fn consume(&self, body: &TicketBody, now: u64) -> Result<(), TicketError> {
        let mut seen = self.seen.lock().map_err(|_| TicketError::Replayed)?;
        seen.retain(|_, exp| *exp > now);
        if seen.contains_key(&body.n) {
            return Err(TicketError::Replayed);
        }
        seen.insert(body.n.clone(), body.exp);
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DirectoryNode {
    pub id: String,
    pub pk: String,
    pub host: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DirectoryParams {
    pub r: u64,
    pub quorum: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DirectoryBody {
    pub v: u64,
    pub epoch: u64,
    pub epoch_seconds: u64,
    pub issued_at: u64,
    pub params: DirectoryParams,
    pub nodes: Vec<DirectoryNode>,
}

/// Ověří obálku adresáře `{body, sig, kid}` klíčem Workeru.
pub fn verify_directory(worker_key: &[u8; 32], envelope: &Value) -> Option<DirectoryBody> {
    let body = envelope.get("body")?;
    let sig: [u8; 64] = b64u_decode_array(envelope.get("sig")?.as_str()?)?;
    let payload = canonical_bytes(body).ok()?;
    if !sign::verify(worker_key, DOMAIN_DIRECTORY, &payload, &sig) {
        return None;
    }
    let dir: DirectoryBody = serde_json::from_value(body.clone()).ok()?;
    (dir.v == 1).then_some(dir)
}
