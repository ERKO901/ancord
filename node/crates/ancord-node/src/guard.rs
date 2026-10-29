//! Stráž schránky (`spec/PROTOCOL.md` §15.13): zápis do schránky vyžaduje razítko proof-of-work.
//!
//! `inbox_id` je odvozené z veřejné adresy, takže do schránky může psát kdokoli. Majitel proto na jejích anchorech
//! zapne stráž (podepsanou svým klíčem) a anchory pak nové zprávy bez platného razítka odmítnou. Stráž cizího mailboxu
//! nikdo zapnout nemůže: `inbox_id` je hash `pk` a podpis musí sedět k `pk`.

use std::collections::BTreeSet;
use std::sync::Arc;

use ed25519_dalek::SigningKey;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::data::{anchors_in, blocking, fan_out, local, peer_post, plan, DataError};
use crate::encoding::{b64u_decode_array, b64u_encode};
use crate::hashcash;
use crate::sign::{self, DOMAIN_INBOX_GUARD};
use crate::state::NodeState;
use crate::store::{GuardRecord, PutOutcome};
use crate::ticket::DirectoryNode;
use crate::time::unix_now;
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

    pub fn check(&self, now: u64) -> Result<(), DataError> {
        if now.abs_diff(self.ts) > TS_SKEW_S {
            return Err(DataError::BadRequest("ts is out of range"));
        }
        if self.bits > MAX_BITS {
            return Err(DataError::BadRequest("bits too high"));
        }
        if !sign::verify(&self.pk, DOMAIN_INBOX_GUARD, &payload(&self.inbox(), self.bits, self.ts), &self.sig) {
            return Err(DataError::InvalidProof);
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

    fn to_record(&self, exp: u64) -> GuardRecord {
        GuardRecord { inbox_id: self.inbox(), pk: self.pk, bits: self.bits, ts: self.ts, sig: self.sig, exp }
    }

    fn from_record(r: &GuardRecord) -> Self {
        Self { pk: r.pk, bits: r.bits, ts: r.ts, sig: r.sig }
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

/// Obtížnost, kterou anchor vyžaduje pro nový zápis do `mailbox_id` (0 = mailbox není střežený).
pub async fn required_bits(st: &Arc<NodeState>, mailbox_id: [u8; 32]) -> Result<u8, DataError> {
    let now = unix_now();
    let rec = blocking(st, move |s| s.store.guard(&mailbox_id, now)).await?;
    Ok(rec.map_or(0, |g| g.bits))
}

pub async fn put_local(st: &Arc<NodeState>, guard: &Guard) -> Result<PutOutcome, DataError> {
    let now = unix_now();
    guard.check(now)?;
    let record = guard.to_record(now.saturating_add(st.cfg.profile_ttl_s));
    let max = st.cfg.max_profiles;
    blocking(st, move |s| s.store.put_guard(&record, max)).await
}

/// Zapne (nebo změní) stráž na anchorech schránky; hotovo po `quorum` potvrzeních.
pub async fn put(st: &Arc<NodeState>, guard: Guard) -> Result<usize, DataError> {
    guard.check(unix_now())?;
    let plan = plan(st, &guard.inbox())?;
    let guard = Arc::new(guard);
    let mut rx = fan_out(st, plan.nodes, move |st, node, is_self| {
        let guard = guard.clone();
        async move {
            if is_self {
                return put_local(&st, &guard).await.map(|_| ()).map_err(local);
            }
            peer_post(&st, &node, "/n/guard", &guard.to_json()).await.map(|_| ())
        }
    });
    let mut acks = 0;
    while let Some(result) = rx.recv().await {
        if result.is_ok() {
            acks += 1;
            if acks >= plan.quorum {
                return Ok(acks);
            }
        }
    }
    Err(DataError::QuorumUnavailable)
}

/// Handoff stráží po změně epochy, stejná pravidla jako u profilů (§15.9).
pub async fn handoff(st: &Arc<NodeState>) -> bool {
    let Some(dir) = st.directory() else { return true };
    let prev = st.prev_directory();
    let me = st.identity.node_id().to_owned();
    let quorum = usize::try_from(dir.params.quorum).unwrap_or(usize::MAX);
    let now = unix_now();
    let Ok(records) = blocking(st, move |s| s.store.guards(now)).await else { return false };
    let (mut all_ok, mut done) = (true, BTreeSet::new());
    for rec in records {
        let current = anchors_in(&dir, &rec.inbox_id);
        let still_anchor = current.iter().any(|n| n.id == me);
        let targets: Vec<DirectoryNode> = current
            .iter()
            .filter(|n| n.id != me)
            .filter(|n| !still_anchor || prev.as_ref().is_none_or(|p| !anchors_in(p, &rec.inbox_id).iter().any(|a| a.id == n.id)))
            .cloned()
            .collect();
        if targets.is_empty() && still_anchor {
            continue;
        }
        let body = Guard::from_record(&rec).to_json();
        let mut accepted = 0;
        for node in &targets {
            if peer_post(st, node, "/n/guard", &body).await.is_ok() {
                accepted += 1;
            }
        }
        if still_anchor {
            all_ok &= accepted == targets.len();
        } else if accepted >= quorum.min(current.len()) {
            done.insert(rec.inbox_id);
        } else {
            all_ok = false;
        }
    }
    let done: Vec<[u8; 32]> = done.into_iter().collect();
    if !done.is_empty() && blocking(st, move |s| s.store.remove_guards(&done)).await.is_err() {
        all_ok = false;
    }
    all_ok
}

