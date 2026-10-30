//! Veřejné profily uživatelů na anchorech (`spec/PROTOCOL.md` §15.2 a §15.3).
//!
//! Profil svazuje adresu uživatele (Ed25519 `pk`) s jeho X25519 klíčem `bk`. Node ho ukládá v čitelné podobě,
//! ověřuje podpis a hashcash, a při čtení vrací nejnovější. Důvěřovat mu nemusí nikdo: klient podpis ověřuje sám.

use ed25519_dalek::SigningKey;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::error::ProofError;
use crate::encoding::{b64u_decode_array, b64u_encode};
use crate::hashcash;
use crate::sign::{self, DOMAIN_PROFILE};
use crate::wake::{parse_push_id, PUSH_ID_LEN};

/// Jak moc se smí `ts` profilu lišit od času nodu.
const TS_SKEW_S: u64 = 86_400;
/// Nejvýše tolik jednorázových prekeys v jednom profilu.
pub const MAX_OTKS: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    pub pk: [u8; 32],
    /// Dlouhodobý X25519 klíč uživatele (identita pro dohodu tajemství).
    pub bk: [u8; 32],
    /// Podepsaný krátkodobý prekey; uživatel ho pravidelně rotuje a starý tajný klíč zahodí.
    pub spk: [u8; 32],
    /// Jednorázové prekeys; příjemce tajný klíč po použití smaže.
    pub otks: Vec<[u8; 32]>,
    /// `push_id` schránky (§15.11): odesílatel ho přiloží k obálce, aby anchor offline příjemce vzbudil.
    pub push_id: String,
    pub ts: u64,
    pub pow: u64,
    pub sig: [u8; 64],
}

/// `sha256("ancord/v1/profile-id" ‖ 0x00 ‖ pk)`: klíč, podle kterého se počítají anchory profilu.
pub fn profile_id(pk: &[u8; 32]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"ancord/v1/profile-id");
    h.update([0]);
    h.update(pk);
    h.finalize().into()
}

/// Podepsaná část profilu: `bk ‖ spk ‖ n ‖ otks ‖ ts_be_u64 ‖ push_id`. Všechno, co klient z profilu použije, je pod
/// podpisem.
fn signed_payload(bk: &[u8; 32], spk: &[u8; 32], otks: &[[u8; 32]], ts: u64, push_id: &str) -> Vec<u8> {
    let mut v = bk.to_vec();
    v.extend_from_slice(spk);
    v.push(u8::try_from(otks.len()).unwrap_or(u8::MAX));
    for otk in otks {
        v.extend_from_slice(otk);
    }
    v.extend_from_slice(&ts.to_be_bytes());
    v.extend_from_slice(push_id.as_bytes());
    v
}

fn pow_nonce(payload: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"ancord/v1/profile-pow");
    h.update([0]);
    h.update(payload);
    h.finalize().into()
}

impl Profile {
    /// Podepíše profil a vyřeší hashcash (`bits` je obtížnost, kterou žádá node).
    pub fn create(uk: &SigningKey, bk: &[u8; 32], spk: &[u8; 32], otks: &[[u8; 32]], push_id: &str, ts: u64, bits: u32) -> Self {
        let pk = uk.verifying_key().to_bytes();
        let payload = signed_payload(bk, spk, otks, ts, push_id);
        let sig = sign::sign(uk, DOMAIN_PROFILE, &payload);
        let threads = std::thread::available_parallelism().map_or(1, usize::from);
        let pow = hashcash::solve(&pow_nonce(&payload), &pk, bits, threads);
        Self { pk, bk: *bk, spk: *spk, otks: otks.to_vec(), push_id: push_id.to_owned(), ts, pow, sig }
    }

    fn payload(&self) -> Vec<u8> {
        signed_payload(&self.bk, &self.spk, &self.otks, self.ts, &self.push_id)
    }

    /// Podpis klíčem `pk` (stačí klientovi, který hashcash a čas neřeší).
    pub fn signature_valid(&self) -> bool {
        self.otks.len() <= MAX_OTKS
            && parse_push_id(&self.push_id).is_ok()
            && sign::verify(&self.pk, DOMAIN_PROFILE, &self.payload(), &self.sig)
    }

    /// Plné ověření na nodu: čas, hashcash a podpis.
    pub fn check(&self, now: u64, bits: u32) -> Result<(), ProofError> {
        if now.abs_diff(self.ts) > TS_SKEW_S {
            return Err(ProofError::BadRequest("ts is out of range"));
        }
        parse_push_id(&self.push_id)?;
        if !hashcash::valid(&pow_nonce(&self.payload()), &self.pk, self.pow, bits) || !self.signature_valid() {
            return Err(ProofError::InvalidProof);
        }
        Ok(())
    }

    pub fn to_json(&self) -> Value {
        let otks: Vec<String> = self.otks.iter().map(|k| b64u_encode(k)).collect();
        json!({
            "pk": b64u_encode(&self.pk), "bk": b64u_encode(&self.bk), "spk": b64u_encode(&self.spk), "otks": otks,
            "push_id": self.push_id, "ts": self.ts, "pow": self.pow, "sig": b64u_encode(&self.sig),
        })
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        Some(Self {
            pk: b64u_decode_array(v["pk"].as_str()?)?,
            bk: b64u_decode_array(v["bk"].as_str()?)?,
            spk: b64u_decode_array(v["spk"].as_str()?)?,
            otks: v["otks"].as_array()?.iter().map(|k| k.as_str().and_then(b64u_decode_array)).collect::<Option<Vec<_>>>().filter(|o| o.len() <= MAX_OTKS)?,
            push_id: v["push_id"].as_str().filter(|p| p.len() == PUSH_ID_LEN)?.to_owned(),
            ts: v["ts"].as_u64()?,
            pow: v["pow"].as_u64()?,
            sig: b64u_decode_array(v["sig"].as_str()?)?,
        })
    }

}

pub fn parse_pk(s: &str) -> Result<[u8; 32], ProofError> {
    b64u_decode_array(s).ok_or(ProofError::BadRequest("invalid pk"))
}

/// Je profil v souladu s očekávanou adresou?
pub fn is_for(profile: &Profile, pk: &[u8; 32]) -> bool {
    &profile.pk == pk && profile.signature_valid()
}
