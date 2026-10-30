//! `push_id` push klíče (spec §10.1 a §15.11), sdílené s klientem.

use crate::error::ProofError;

pub const PUSH_ID_LEN: usize = 52;

/// `push_id` push klíče: `base32(blake3(push_pubkey))` (spec §10.1).
pub fn push_id_of(push_pubkey: &[u8; 32]) -> String {
    crate::encoding::base32_encode(blake3::hash(push_pubkey).as_bytes())
}

/// `push_id` je `base32(blake3(push_pubkey))`: 52 znaků `[a-z2-7]`.
pub fn parse_push_id(s: &str) -> Result<String, ProofError> {
    if s.len() == PUSH_ID_LEN && s.bytes().all(|b| matches!(b, b'a'..=b'z' | b'2'..=b'7')) {
        Ok(s.to_owned())
    } else {
        Err(ProofError::BadRequest("push_id"))
    }
}

/// Nepovinné `push_id` z JSON těla; chybějící nebo `null` je `None`, špatný tvar je chyba.
pub fn optional_push_id(v: &serde_json::Value) -> Result<Option<String>, ProofError> {
    match v {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::String(s) => parse_push_id(s).map(Some),
        _ => Err(ProofError::BadRequest("push_id")),
    }
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
        assert!(optional_push_id(&serde_json::json!(5)).is_err());
    }
}
