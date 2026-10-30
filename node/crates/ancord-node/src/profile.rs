//! Veřejné profily na anchorech: ukládání, replikace a handoff (`spec/PROTOCOL.md` §15.2 a §15.3).
//! Typ `Profile` a jeho ověření jsou v `ancord-core` (sdílí je klient).

use std::collections::BTreeSet;
use std::sync::Arc;

use serde_json::{json, Value};

use crate::data::{anchors_in, blocking, fan_out, local, peer_post, plan, DataError, OpError};
use crate::encoding::b64u_encode;
use crate::state::NodeState;
use crate::store::{ProfileRecord, PutOutcome};
use crate::ticket::DirectoryNode;
use crate::time::unix_now;

pub use ancord_core::profile::*;

fn to_record(p: &Profile, exp: u64) -> ProfileRecord {
    ProfileRecord { pk: p.pk, bk: p.bk, spk: p.spk, otks: p.otks.clone(), push_id: p.push_id.clone(), ts: p.ts, pow: p.pow, sig: p.sig, exp }
}

fn from_record(r: &ProfileRecord) -> Profile {
    Profile { pk: r.pk, bk: r.bk, spk: r.spk, otks: r.otks.clone(), push_id: r.push_id.clone(), ts: r.ts, pow: r.pow, sig: r.sig }
}

/// Zápis do vlastního úložiště; profil se předem ověří.
pub async fn put_local(st: &Arc<NodeState>, profile: &Profile) -> Result<PutOutcome, DataError> {
    let now = unix_now();
    profile.check(now, st.cfg.profile_pow_bits)?;
    let record = to_record(profile, now.saturating_add(st.cfg.profile_ttl_s));
    let max = st.cfg.max_profiles;
    blocking(st, move |s| s.store.put_profile(&record, max)).await
}

pub async fn get_local(st: &Arc<NodeState>, pk: [u8; 32]) -> Result<Option<Profile>, DataError> {
    let now = unix_now();
    let record = blocking(st, move |s| s.store.profile(&pk, now)).await?;
    Ok(record.as_ref().map(from_record))
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
            Err(OpError::PowRequired(_)) => {}
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
        let body = from_record(&rec).to_json();
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
