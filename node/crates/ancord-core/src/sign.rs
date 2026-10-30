//! Podpisy Ed25519 s doménovou separací a podepsané HTTP requesty (`spec/PROTOCOL.md` §1.3 a §3).

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};

use crate::encoding::{b64u_decode_array, b64u_encode};

pub const DOMAIN_REGISTER: &str = "ancord/v1/register";
pub const DOMAIN_PROOF: &str = "ancord/v1/proof";
pub const DOMAIN_REQ: &str = "ancord/v1/req";
pub const DOMAIN_HEARTBEAT: &str = "ancord/v1/heartbeat";
pub const DOMAIN_DIRECTORY: &str = "ancord/v1/directory";
pub const DOMAIN_TICKET: &str = "ancord/v1/ticket";
pub const DOMAIN_REPORT: &str = "ancord/v1/report";
pub const DOMAIN_CHAN_NODE: &str = "ancord/v1/chan-node";
pub const DOMAIN_CHAN_CLIENT: &str = "ancord/v1/chan-client";
pub const DOMAIN_N2N_RESP: &str = "ancord/v1/n2n-resp";
pub const DOMAIN_PUSH_REG: &str = "ancord/v1/push-reg";
pub const DOMAIN_PROFILE: &str = "ancord/v1/profile";
pub const DOMAIN_INBOX_GUARD: &str = "ancord/v1/inbox-guard";

/// `DOMAIN ‖ 0x00 ‖ payload`.
pub fn signing_input(domain: &str, payload: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(domain.len() + 1 + payload.len());
    v.extend_from_slice(domain.as_bytes());
    v.push(0);
    v.extend_from_slice(payload);
    v
}

pub fn sign(key: &SigningKey, domain: &str, payload: &[u8]) -> [u8; 64] {
    key.sign(&signing_input(domain, payload)).to_bytes()
}

/// Ověří podpis; neplatný vstup je `false`, nikdy panika.
pub fn verify(pubkey: &[u8; 32], domain: &str, payload: &[u8], sig: &[u8; 64]) -> bool {
    let Ok(vk) = VerifyingKey::from_bytes(pubkey) else {
        return false;
    };
    vk.verify(&signing_input(domain, payload), &Signature::from_bytes(sig)).is_ok()
}

pub fn sha256(data: &[u8]) -> [u8; 32] {
    Sha256::digest(data).into()
}

/// Payload podepsaného requestu: `METHOD \n PATH_AND_QUERY \n t \n n \n b64u(sha256(body))`.
pub fn request_payload(method: &str, path_and_query: &str, t: u64, n: &str, body: &[u8]) -> Vec<u8> {
    format!("{method}\n{path_and_query}\n{t}\n{n}\n{}", b64u_encode(&sha256(body))).into_bytes()
}

/// Hlavička `Authorization: Ancord-Ed25519 k=…,t=…,n=…,s=…`.
pub fn auth_header(key: &SigningKey, method: &str, path_and_query: &str, body: &[u8], t: u64, nonce: &[u8; 16]) -> String {
    let n = b64u_encode(nonce);
    let sig = sign(key, DOMAIN_REQ, &request_payload(method, path_and_query, t, &n, body));
    format!(
        "Ancord-Ed25519 k={},t={t},n={n},s={}",
        b64u_encode(key.verifying_key().as_bytes()),
        b64u_encode(&sig)
    )
}

/// Rozebraná hlavička podepsaného requestu.
#[derive(Debug, PartialEq, Eq)]
pub struct ParsedAuth {
    pub pubkey: [u8; 32],
    pub t: u64,
    pub nonce_b64: String,
    pub sig: [u8; 64],
}

pub fn parse_auth_header(header: &str) -> Option<ParsedAuth> {
    let rest = header.strip_prefix("Ancord-Ed25519 ")?;
    let (mut k, mut t, mut n, mut s) = (None, None, None, None);
    for part in rest.split(',') {
        let (name, value) = part.split_once('=')?;
        match name {
            "k" => k = Some(value),
            "t" => t = Some(value),
            "n" => n = Some(value),
            "s" => s = Some(value),
            _ => return None,
        }
    }
    let nonce = n?;
    b64u_decode_array::<16>(nonce)?;
    Some(ParsedAuth {
        pubkey: b64u_decode_array(k?)?,
        t: t?.parse().ok()?,
        nonce_b64: nonce.to_owned(),
        sig: b64u_decode_array(s?)?,
    })
}

/// Ověří podepsaný request (formát, časové okno, podpis). Replay řeší volající.
pub fn verify_request(
    header: &str,
    method: &str,
    path_and_query: &str,
    body: &[u8],
    now: u64,
    skew_s: u64,
) -> Option<ParsedAuth> {
    let auth = parse_auth_header(header)?;
    if now.abs_diff(auth.t) > skew_s {
        return None;
    }
    let payload = request_payload(method, path_and_query, auth.t, &auth.nonce_b64, body);
    verify(&auth.pubkey, DOMAIN_REQ, &payload, &auth.sig).then_some(auth)
}
