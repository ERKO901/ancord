//! Veřejné profily uživatelů na anchorech (`spec/PROTOCOL.md` §15.2 a §15.3).
//!
//! Profil svazuje adresu uživatele (Ed25519 `pk`) s jeho X25519 klíčem `bk`. Node ho ukládá v čitelné podobě,
//! ověřuje podpis a hashcash, a při čtení vrací nejnovější. Důvěřovat mu nemusí nikdo: klient podpis ověřuje sám.

use std::collections::BTreeSet;
use std::sync::Arc;

use ed25519_dalek::SigningKey;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::data::{anchors_in, blocking, fan_out, local, peer_post, plan, DataError, OpError};
use crate::encoding::{b64u_decode_array, b64u_encode};
use crate::hashcash;
use crate::sign::{self, DOMAIN_PROFILE};
use crate::state::NodeState;
use crate::store::{ProfileRecord, PutOutcome};
use crate::ticket::DirectoryNode;
use crate::time::unix_now;
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
    pub fn check(&self, now: u64, bits: u32) -> Result<(), DataError> {
        if now.abs_diff(self.ts) > TS_SKEW_S {
            return Err(DataError::BadRequest("ts is out of range"));
        }
        parse_push_id(&self.push_id)?;
        if !hashcash::valid(&pow_nonce(&self.payload()), &self.pk, self.pow, bits) || !self.signature_valid() {
            return Err(DataError::InvalidProof);
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

    fn to_record(&self, exp: u64) -> ProfileRecord {
        ProfileRecord { pk: self.pk, bk: self.bk, spk: self.spk, otks: self.otks.clone(), push_id: self.push_id.clone(), ts: self.ts, pow: self.pow, sig: self.sig, exp }
    }

    fn from_record(r: &ProfileRecord) -> Self {
        Self { pk: r.pk, bk: r.bk, spk: r.spk, otks: r.otks.clone(), push_id: r.push_id.clone(), ts: r.ts, pow: r.pow, sig: r.sig }
    }
}

pub fn parse_pk(s: &str) -> Result<[u8; 32], DataError> {
    b64u_decode_array(s).ok_or(DataError::BadRequest("invalid pk"))
}

/// Zápis do vlastního úložiště; profil se předem ověří.
pub async fn put_local(st: &Arc<NodeState>, profile: &Profile) -> Result<PutOutcome, DataError> {
    let now = unix_now();
    profile.check(now, st.cfg.profile_pow_bits)?;
    let record = profile.to_record(now.saturating_add(st.cfg.profile_ttl_s));
    let max = st.cfg.max_profiles;
    blocking(st, move |s| s.store.put_profile(&record, max)).await
}

pub async fn get_local(st: &Arc<NodeState>, pk: [u8; 32]) -> Result<Option<Profile>, DataError> {
    let now = unix_now();
    let record = blocking(st, move |s| s.store.profile(&pk, now)).await?;
    Ok(record.as_ref().map(Profile::from_record))
}

/// Zápis profilu na anchory; hotovo po `quorum` potvrzeních. Vrací jejich počet.
pub async fn put(st: &Arc<NodeState>, profile: Profile) -> Result<usize, DataError> {
    profile.check(unix_now(), st.cfg.profile_pow_bits)?;
    let plan = plan(st, &profile_id(&profile.pk))?;
    let profile = Arc::new(profile);
    let mut rx = fan_out(st, plan.nodes, move |st, node, is_self| {
        let profile = profile.clone();
        async move {
            if is_self {
                return put_local(&st, &profile).await.map(|_| ()).map_err(local);
            }
            peer_post(&st, &node, "/n/profile_put", &profile.to_json()).await.map(|_| ())
        }
    });
    let (mut acks, mut full) = (0, false);
    while let Some(result) = rx.recv().await {
        match result {
            Ok(()) => {
                acks += 1;
                if acks >= plan.quorum {
                    return Ok(acks);
                }
            }
            Err(OpError::Full) => full = true,
            Err(OpError::Failed(reason)) => tracing::debug!(%reason, "profile write failed"),
        }
    }
    Err(if full { DataError::MailboxFull } else { DataError::QuorumUnavailable })
}

/// Nejnovější profil z odpovědí nejméně `r − quorum + 1` anchorů.
pub async fn get(st: &Arc<NodeState>, pk: [u8; 32]) -> Result<Profile, DataError> {
    let plan = plan(st, &profile_id(&pk))?;
    let need = plan.read_quorum();
    let mut rx = fan_out(st, plan.nodes, move |st, node, is_self| async move {
        if is_self {
            return get_local(&st, pk).await.map_err(local);
        }
        let answer = peer_post(&st, &node, "/n/profile_get", &json!({"pk": b64u_encode(&pk)})).await?;
        match &answer["profile"] {
            Value::Null => Ok(None),
            v => Profile::from_json(v).filter(|p| p.pk == pk).map(Some).ok_or_else(|| OpError::Failed("malformed profile".into())),
        }
    });
    let (mut answers, mut best): (usize, Option<Profile>) = (0, None);
    while let Some(result) = rx.recv().await {
        if let Ok(found) = result {
            answers += 1;
            // anchor mohl dostat cokoli: vítězí jen to, co má platný podpis
            if let Some(p) = found.filter(Profile::signature_valid) {
                if best.as_ref().is_none_or(|b| p.ts > b.ts) {
                    best = Some(p);
                }
            }
            if answers >= need {
                return best.ok_or(DataError::NotFound);
            }
        }
    }
    Err(DataError::QuorumUnavailable)
}

/// Handoff profilů po změně epochy (`spec/PROTOCOL.md` §15.9), stejná pravidla jako u zpráv (§13.5).
pub async fn handoff(st: &Arc<NodeState>) -> bool {
    let Some(dir) = st.directory() else { return true };
    let prev = st.prev_directory();
    let me = st.identity.node_id().to_owned();
    let quorum = usize::try_from(dir.params.quorum).unwrap_or(usize::MAX);
    let now = unix_now();
    let Ok(records) = blocking(st, move |s| s.store.profiles(now)).await else { return false };
    let (mut all_ok, mut done) = (true, BTreeSet::new());

    for rec in records {
        let id = profile_id(&rec.pk);
        let current = anchors_in(&dir, &id);
        let still_anchor = current.iter().any(|n| n.id == me);
        let targets: Vec<DirectoryNode> = current
            .iter()
            .filter(|n| n.id != me)
            .filter(|n| !still_anchor || prev.as_ref().is_none_or(|p| !anchors_in(p, &id).iter().any(|a| a.id == n.id)))
            .cloned()
            .collect();
        if targets.is_empty() && still_anchor {
            continue;
        }
        let body = Profile::from_record(&rec).to_json();
        let mut accepted = 0;
        for node in &targets {
            if peer_post(st, node, "/n/profile_put", &body).await.is_ok() {
                accepted += 1;
            }
        }
        if still_anchor {
            all_ok &= accepted == targets.len();
        } else if accepted >= quorum.min(current.len()) {
            done.insert(rec.pk);
        } else {
            all_ok = false;
        }
    }
    let done: Vec<[u8; 32]> = done.into_iter().collect();
    if !done.is_empty() && blocking(st, move |s| s.store.remove_profiles(&done)).await.is_err() {
        all_ok = false;
    }
    all_ok
}

/// Pro testy a CLI: je profil v souladu s očekávanou adresou?
pub fn is_for(profile: &Profile, pk: &[u8; 32]) -> bool {
    &profile.pk == pk && profile.signature_valid()
}
