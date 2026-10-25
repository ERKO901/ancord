//! Údržba na pozadí: handoff po změně epochy a úklid vypršených zpráv (`spec/PROTOCOL.md` §13.5).

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use crate::data;
use crate::presence;
use crate::state::NodeState;
use crate::time::unix_now;

/// Jeden krok údržby: handoff, pokud ještě neproběhl pro aktuální epochu. Vrací `true`, když něco spustil.
pub async fn handoff_if_needed(st: &Arc<NodeState>) -> bool {
    let Some(epoch) = st.epoch() else { return false };
    if st.handoff_epoch.load(Ordering::Relaxed) == epoch {
        return false;
    }
    if data::handoff(st).await {
        st.handoff_epoch.store(epoch, Ordering::Relaxed);
        tracing::info!(epoch, "handoff finished");
    } else {
        tracing::warn!(epoch, "handoff incomplete, will retry");
    }
    true
}

pub async fn purge_expired(st: &Arc<NodeState>) {
    let st2 = st.clone();
    let now = unix_now();
    match tokio::task::spawn_blocking(move || st2.store.purge_expired(now)).await {
        Ok(Ok(0)) => {}
        Ok(Ok(n)) => tracing::info!(removed = n, "expired messages purged"),
        Ok(Err(e)) => tracing::error!(error = %e, "purge failed"),
        Err(e) => tracing::error!(error = %e, "purge task failed"),
    }
}

pub async fn run(st: Arc<NodeState>) {
    let mut tick = tokio::time::interval(Duration::from_secs(st.cfg.maintenance_interval_s.max(1)));
    let janitor_every = Duration::from_secs(st.cfg.janitor_interval_s.max(1));
    let mut last_purge = tokio::time::Instant::now();
    let mut renew = tokio::time::interval(Duration::from_secs(st.cfg.presence_renew_s.max(1)));
    purge_expired(&st).await;
    loop {
        tokio::select! {
            _ = tick.tick() => {
                handoff_if_needed(&st).await;
                if last_purge.elapsed() >= janitor_every {
                    purge_expired(&st).await;
                    last_purge = tokio::time::Instant::now();
                }
            }
            _ = renew.tick() => {
                presence::prune(&st);
                presence::renew_all(&st);
            }
        }
    }
}
