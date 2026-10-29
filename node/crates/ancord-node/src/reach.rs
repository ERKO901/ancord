//! Dosažitelnost peerů z pohledu tohoto nodu (`spec/PROTOCOL.md` §6.1) a stažení novějšího adresáře od peera.
//!
//! Neúspěšná volání se počítají po nodech; peer s `reach_min_failures` neúspěchy po sobě se hlásí v heartbeatu
//! (`unreachable`). Hlášení je jen podnět: Worker dosažitelnost ověří sám.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use serde_json::Value;

use crate::state::NodeState;
use crate::ticket::DirectoryNode;
use crate::time::unix_now;

/// Nejvýše tolik nodů v jednom heartbeatu (Worker víc odmítne).
pub const MAX_REPORTED: usize = 16;
/// Nejkratší odstup dvou stažení adresáře od peerů.
const PULL_MIN_INTERVAL_S: u64 = 10;

#[derive(Default)]
struct PeerHealth {
    fails: u32,
    last_fail: u64,
}

#[derive(Default)]
pub struct Reach {
    peers: Mutex<HashMap<String, PeerHealth>>,
    last_pull: AtomicU64,
}

/// Odpovědi, které znamenají, že se request k nodu nedostal: Cloudflare nedosáhl originu (tunel, `cloudflared`).
pub fn is_transport_status(status: u16) -> bool {
    matches!(status, 502 | 504 | 520..=530)
}

impl Reach {
    pub fn record(&self, node_id: &str, ok: bool, now: u64) {
        let Ok(mut peers) = self.peers.lock() else { return };
        if ok {
            peers.remove(node_id);
            return;
        }
        let h = peers.entry(node_id.to_owned()).or_default();
        h.fails = h.fails.saturating_add(1);
        h.last_fail = now;
    }

    /// Peeři k nahlášení (seřazení, nejvýše `MAX_REPORTED`); staré záznamy zahodí.
    pub fn unreachable(&self, now: u64, min_fails: u32, window_s: u64) -> Vec<String> {
        let Ok(mut peers) = self.peers.lock() else { return Vec::new() };
        peers.retain(|_, h| now.saturating_sub(h.last_fail) <= window_s);
        let mut out: Vec<String> = peers.iter().filter(|(_, h)| h.fails >= min_fails.max(1)).map(|(id, _)| id.clone()).collect();
        out.sort();
        out.truncate(MAX_REPORTED);
        out
    }

    fn may_pull(&self, now: u64) -> bool {
        let last = self.last_pull.load(Ordering::Relaxed);
        now.saturating_sub(last) >= PULL_MIN_INTERVAL_S
            && self.last_pull.compare_exchange(last, now, Ordering::Relaxed, Ordering::Relaxed).is_ok()
    }
}

/// Peer odmítl požadavek (`403`): možná drží novější adresář než my (Worker nám ho nestihl poslat, nebo je
/// nedostupný). Adresář je podepsaný Workerem, takže ho lze převzít od kohokoli. Vrací `true`, když byl novější.
pub async fn pull_directory(st: &NodeState, node: &DirectoryNode) -> bool {
    if !st.reach.may_pull(unix_now()) {
        return false;
    }
    let url = crate::n2n::url(st, node, "/n/directory");
    let Ok(res) = st.peer_http.get(url).send().await else { return false };
    if !res.status().is_success() {
        return false;
    }
    let Ok(envelope) = res.json::<Value>().await else { return false };
    match st.accept_directory(&envelope) {
        Some(epoch) => {
            tracing::info!(epoch, from = %node.id, "directory updated from peer");
            true
        }
        None => false,
    }
}
