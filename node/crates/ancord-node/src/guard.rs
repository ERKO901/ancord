//! Stráž schránky na anchorech: ukládání, replikace a handoff (`spec/PROTOCOL.md` §15.13).
//! Typ `Guard` a razítko proof-of-work jsou v `ancord-core` (sdílí je klient).

use std::collections::BTreeSet;
use std::sync::Arc;


use crate::data::{anchors_in, blocking, fan_out, local, peer_post, plan, DataError};
use crate::state::NodeState;
use crate::store::{GuardRecord, PutOutcome};
use crate::ticket::DirectoryNode;
use crate::time::unix_now;

pub use ancord_core::guard::*;

fn to_record(g: &Guard, exp: u64) -> GuardRecord {
    GuardRecord { inbox_id: g.inbox(), pk: g.pk, bits: g.bits, ts: g.ts, sig: g.sig, exp }
}

fn from_record(r: &GuardRecord) -> Guard {
    Guard { pk: r.pk, bits: r.bits, ts: r.ts, sig: r.sig }
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
    let record = to_record(guard, now.saturating_add(st.cfg.profile_ttl_s));
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
        let body = from_record(&rec).to_json();
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

