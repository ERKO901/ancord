//! Push wake: anchor, který nově uložil zprávu, upozorní Worker (`spec/PROTOCOL.md` §10.2 a §13.6).

use std::sync::Arc;

use reqwest::Method;
use serde_json::json;

use crate::data::DataError;
use crate::encoding::b64u_encode;
use crate::state::NodeState;
use crate::time::unix_now;

pub const PUSH_ID_LEN: usize = 52;

/// `push_id` je `base32(blake3(push_pubkey))`: 52 znaků `[a-z2-7]`.
pub fn parse_push_id(s: &str) -> Result<String, DataError> {
    if s.len() == PUSH_ID_LEN && s.bytes().all(|b| matches!(b, b'a'..=b'z' | b'2'..=b'7')) {
        Ok(s.to_owned())
    } else {
        Err(DataError::BadRequest("push_id"))
    }
}

/// Nepovinné `push_id` z JSON těla; chybějící nebo `null` je `None`, špatný tvar je chyba.
pub fn optional_push_id(v: &serde_json::Value) -> Result<Option<String>, DataError> {
    match v {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::String(s) => parse_push_id(s).map(Some),
        _ => Err(DataError::BadRequest("push_id")),
    }
}

/// Lokální koalescence: `true`, když se pro `push_id` smí wake poslat teď.
fn admit(st: &NodeState, push_id: &str, now: u64) -> bool {
    let Ok(mut last) = st.wake_last.lock() else {
        return false;
    };
    let interval = st.cfg.wake_min_interval_s;
    last.retain(|_, at| at.saturating_add(interval) > now);
    if last.contains_key(push_id) {
        return false;
    }
    last.insert(push_id.to_owned(), now);
    true
}

/// Pošle wake na pozadí. Selhání se jen zaloguje, zápis zprávy se tím nikdy neruší.
pub fn spawn(st: &Arc<NodeState>, push_id: String, mailbox_id: [u8; 32]) {
    if !admit(st, &push_id, unix_now()) {
        return;
    }
    let st = st.clone();
    tokio::spawn(async move {
        let body = json!({"push_id": push_id, "mailbox_id": b64u_encode(&mailbox_id)});
        match st.worker.signed(st.identity.signing_key(), Method::POST, "/wake", Some(&body)).await {
            Ok(answer) => tracing::debug!(status = answer["status"].as_str().unwrap_or("?"), "wake sent"),
            Err(e) => tracing::warn!(error = %e, "wake failed"),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_id_shape() {
        assert!(parse_push_id(&"a".repeat(52)).is_ok());
        assert!(parse_push_id(&"A".repeat(52)).is_err());
        assert!(parse_push_id(&"a".repeat(51)).is_err());
        assert!(parse_push_id(&"1".repeat(52)).is_err());
        assert_eq!(optional_push_id(&serde_json::Value::Null), Ok(None));
        assert!(optional_push_id(&json!(5)).is_err());
    }
}
