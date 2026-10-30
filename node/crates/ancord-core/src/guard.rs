//! Stráž schránky (`spec/PROTOCOL.md` §15.13): zápis do schránky vyžaduje razítko proof-of-work.
//!
//! `inbox_id` je odvozené z veřejné adresy, takže do schránky může psát kdokoli. Majitel proto na jejích anchorech
//! zapne stráž (podepsanou svým klíčem) a anchory pak nové zprávy bez platného razítka odmítnou. Stráž cizího mailboxu
//! nikdo zapnout nemůže: `inbox_id` je hash `pk` a podpis musí sedět k `pk`.

use ed25519_dalek::SigningKey;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::error::ProofError;
use crate::encoding::{b64u_decode_array, b64u_encode};
use crate::hashcash;
use crate::sign::{self, DOMAIN_INBOX_GUARD};
use crate::user::inbox_id;

/// Nejvyšší obtížnost, kterou si majitel může vyžádat (víc by odesílatele odřízlo).
pub const MAX_BITS: u8 = 28;
/// Délka razítka na začátku `ct`: čítač hashcash (u64 LE).
pub const STAMP_LEN: usize = 8;
const TS_SKEW_S: u64 = 86_400;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Guard {
    pub pk: [u8; 32],
    pub bits: u8,
    pub ts: u64,
    pub sig: [u8; 64],
}

/// Podepsaná část: `inbox_id(32) ‖ bits(1) ‖ ts_be_u64`.
fn payload(inbox: &[u8; 32], bits: u8, ts: u64) -> Vec<u8> {
    let mut v = inbox.to_vec();
    v.push(bits);
    v.extend_from_slice(&ts.to_be_bytes());
    v
}

impl Guard {
    pub fn create(uk: &SigningKey, bits: u8, ts: u64) -> Self {
        let pk = uk.verifying_key().to_bytes();
        let sig = sign::sign(uk, DOMAIN_INBOX_GUARD, &payload(&inbox_id(&pk), bits, ts));
        Self { pk, bits, ts, sig }
    }

    pub fn inbox(&self) -> [u8; 32] {
        inbox_id(&self.pk)
    }

    pub fn check(&self, now: u64) -> Result<(), ProofError> {
        if now.abs_diff(self.ts) > TS_SKEW_S {
            return Err(ProofError::BadRequest("ts is out of range"));
        }
        if self.bits > MAX_BITS {
            return Err(ProofError::BadRequest("bits too high"));
        }
        if !sign::verify(&self.pk, DOMAIN_INBOX_GUARD, &payload(&self.inbox(), self.bits, self.ts), &self.sig) {
            return Err(ProofError::InvalidProof);
        }
        Ok(())
    }

    pub fn to_json(&self) -> Value {
        json!({"pk": b64u_encode(&self.pk), "bits": self.bits, "ts": self.ts, "sig": b64u_encode(&self.sig)})
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        Some(Self {
            pk: b64u_decode_array(v["pk"].as_str()?)?,
            bits: u8::try_from(v["bits"].as_u64()?).ok()?,
            ts: v["ts"].as_u64()?,
            sig: b64u_decode_array(v["sig"].as_str()?)?,
        })
    }

}

/// `sha256("ancord/v1/inbox-pow" ‖ 0x00 ‖ mailbox_id ‖ id ‖ obálka)`: razítko platí jen pro tuto zprávu v této schránce.
fn stamp_nonce(mailbox_id: &[u8; 32], id: &[u8; 16], envelope: &[u8]) -> [u8; 32] {
    Sha256::new().chain_update(b"ancord/v1/inbox-pow\0").chain_update(mailbox_id).chain_update(id).chain_update(envelope).finalize().into()
}

/// Platí razítko na začátku `ct` (`pow_le_u64 ‖ obálka`) pro obtížnost `bits`?
pub fn stamp_valid(mailbox_id: &[u8; 32], id: &[u8; 16], ct: &[u8], bits: u8) -> bool {
    let Some((counter, envelope)) = ct.split_first_chunk::<STAMP_LEN>() else { return false };
    hashcash::valid(&stamp_nonce(mailbox_id, id, envelope), mailbox_id, u64::from_le_bytes(*counter), u32::from(bits))
}

/// Vyřeší razítko a vrátí `ct = pow_le_u64 ‖ obálka`.
pub fn stamp(mailbox_id: &[u8; 32], id: &[u8; 16], envelope: &[u8], bits: u8) -> Vec<u8> {
    let threads = std::thread::available_parallelism().map_or(1, usize::from);
    let counter = hashcash::solve(&stamp_nonce(mailbox_id, id, envelope), mailbox_id, u32::from(bits), threads);
    let mut ct = counter.to_le_bytes().to_vec();
    ct.extend_from_slice(envelope);
    ct
}

/// Obálka bez razítka (příjemce): `None`, když je `ct` kratší než razítko.
pub fn strip_stamp(ct: &[u8]) -> Option<&[u8]> {
    ct.get(STAMP_LEN..)
}
