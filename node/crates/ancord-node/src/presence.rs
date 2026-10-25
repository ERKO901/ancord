//! Presence a online doručení (`spec/PROTOCOL.md` §13.7).
//!
//! Gateway (node, na který je klient připojen) po `subscribe` oznámí anchorům mailboxu, že je mailbox online u něj.
//! Anchor, který pak nově uloží zprávu, ji přepošle na gateway; wake se posílá, jen když doručení online selhalo.

use std::collections::VecDeque;
use std::sync::Arc;

use serde_json::{json, Value};
use tokio::sync::mpsc;

use crate::data::{self, fan_out, peer_post};
use crate::encoding::b64u_encode;
use crate::state::NodeState;
use crate::time::unix_now;
use crate::wake;

/// Kolik mailboxů smí odebírat jedno spojení.
pub const MAX_SUBSCRIPTIONS: usize = 32;
/// Strop počtu presence záznamů na anchoru.
const MAX_PRESENCE: usize = 200_000;
/// Kolik posledních `id` si gateway pamatuje kvůli duplicitám od více anchorů.
const RECENT_IDS: usize = 64;

/// Kde je mailbox online (na anchoru).
#[derive(Debug, Clone)]
pub struct PresenceEntry {
    gateway: String,
    exp: u64,
}

struct Subscriber {
    conn: u64,
    tx: mpsc::Sender<Value>,
}

/// Odběratelé jednoho mailboxu na gateway.
#[derive(Default)]
pub struct Subscribers {
    conns: Vec<Subscriber>,
    recent: VecDeque<[u8; 16]>,
}

// --- gateway ---------------------------------------------------------------------------------------------------

/// Zaregistruje odběr. `false`, když se nepodařilo zamknout stav.
pub fn add_subscriber(st: &NodeState, conn: u64, tx: mpsc::Sender<Value>, mailbox: [u8; 32]) -> bool {
    let Ok(mut subs) = st.subscribers.lock() else { return false };
    let entry = subs.entry(mailbox).or_default();
    if !entry.conns.iter().any(|s| s.conn == conn) {
        entry.conns.push(Subscriber { conn, tx });
    }
    true
}

/// Zruší odběr. `true`, když mailbox nemá žádného dalšího odběratele (presence se má zrušit).
pub fn remove_subscriber(st: &NodeState, conn: u64, mailbox: [u8; 32]) -> bool {
    let Ok(mut subs) = st.subscribers.lock() else { return false };
    let Some(entry) = subs.get_mut(&mailbox) else { return false };
    entry.conns.retain(|s| s.conn != conn);
    if entry.conns.is_empty() {
        subs.remove(&mailbox);
        return true;
    }
    false
}

/// Zruší všechny odběry spojení; vrací mailboxy, které tím zůstaly bez odběratele.
pub fn remove_conn(st: &NodeState, conn: u64) -> Vec<[u8; 32]> {
    let Ok(mut subs) = st.subscribers.lock() else { return Vec::new() };
    let mut orphaned = Vec::new();
    subs.retain(|mailbox, entry| {
        entry.conns.retain(|s| s.conn != conn);
        let keep = !entry.conns.is_empty();
        if !keep {
            orphaned.push(*mailbox);
        }
        keep
    });
    orphaned
}

/// Oznámí (`active`) nebo zruší presence mailboxu u jeho anchorů. Čeká na kvorum potvrzení; vrací jejich počet.
pub async fn announce(st: &Arc<NodeState>, mailbox: [u8; 32], active: bool) -> usize {
    let Some(dir) = st.directory() else { return 0 };
    let nodes = data::anchors_in(&dir, &mailbox);
    let need = usize::try_from(dir.params.quorum).unwrap_or(usize::MAX).clamp(1, nodes.len().max(1));
    let exp = if active { unix_now().saturating_add(st.cfg.presence_ttl_s) } else { 0 };
    let mut rx = fan_out(st, nodes, move |st, node, is_self| async move {
        if is_self {
            let me = st.identity.node_id().to_owned();
            set_presence(&st, mailbox, &me, exp);
            return Ok(());
        }
        peer_post(&st, &node, "/n/presence", &json!({"mailbox_id": b64u_encode(&mailbox), "exp": exp})).await.map(|_| ())
    });
    let mut acks = 0;
    while let Some(result) = rx.recv().await {
        if result.is_ok() {
            acks += 1;
            if acks >= need {
                break;
            }
        }
    }
    acks
}

/// Obnoví presence všech mailboxů, které tento node odebírá. Běží na pozadí.
pub fn renew_all(st: &Arc<NodeState>) {
    let mailboxes: Vec<[u8; 32]> = st.subscribers.lock().map(|s| s.keys().copied().collect()).unwrap_or_default();
    for mailbox in mailboxes {
        let st = st.clone();
        tokio::spawn(async move {
            announce(&st, mailbox, true).await;
        });
    }
}

/// Předá zprávu připojeným odběratelům. `true`, když ji aspoň jeden dostal nebo ji gateway už předala dřív.
pub fn deliver_local(st: &NodeState, mailbox: &[u8; 32], id: [u8; 16], ts: u64, ct: &[u8]) -> bool {
    let Ok(mut subs) = st.subscribers.lock() else { return false };
    let Some(entry) = subs.get_mut(mailbox) else { return false };
    if entry.recent.contains(&id) {
        return true;
    }
    entry.conns.retain(|s| !s.tx.is_closed());
    let msg = json!({"t": "message", "mailbox_id": b64u_encode(mailbox), "id": b64u_encode(&id), "ts": ts, "ct": b64u_encode(ct)});
    let mut sent = false;
    for s in &entry.conns {
        sent |= s.tx.try_send(msg.clone()).is_ok();
    }
    if sent {
        if entry.recent.len() >= RECENT_IDS {
            entry.recent.pop_front();
        }
        entry.recent.push_back(id);
    }
    sent
}

// --- anchor ----------------------------------------------------------------------------------------------------

/// Zapíše (nebo při `exp` v minulosti či 0 zruší) presence mailboxu pro gateway.
pub fn set_presence(st: &NodeState, mailbox: [u8; 32], gateway: &str, exp: u64) {
    let now = unix_now();
    let Ok(mut table) = st.presence.lock() else { return };
    if exp <= now {
        if table.get(&mailbox).is_some_and(|e| e.gateway == gateway) {
            table.remove(&mailbox);
        }
        return;
    }
    let exp = exp.min(now.saturating_add(st.cfg.presence_ttl_s));
    if table.len() >= MAX_PRESENCE && !table.contains_key(&mailbox) {
        table.retain(|_, e| e.exp > now);
        if table.len() >= MAX_PRESENCE {
            return;
        }
    }
    table.insert(mailbox, PresenceEntry { gateway: gateway.to_owned(), exp });
}

/// Odstraní prošlou presence.
pub fn prune(st: &NodeState) {
    let now = unix_now();
    if let Ok(mut table) = st.presence.lock() {
        table.retain(|_, e| e.exp > now);
    }
}

fn current_presence(st: &NodeState, mailbox: &[u8; 32]) -> Option<PresenceEntry> {
    let now = unix_now();
    st.presence.lock().ok()?.get(mailbox).filter(|e| e.exp > now).cloned()
}

/// Pokusí se zprávu doručit příjemci online přes jeho gateway.
async fn deliver_online(st: &Arc<NodeState>, mailbox: &[u8; 32], id: [u8; 16], ts: u64, ct: &[u8]) -> bool {
    let Some(entry) = current_presence(st, mailbox) else { return false };
    if entry.gateway == st.identity.node_id() {
        return deliver_local(st, mailbox, id, ts, ct);
    }
    let Some(node) = data::find_node(st, &entry.gateway) else { return false };
    let body = json!({"mailbox_id": b64u_encode(mailbox), "id": b64u_encode(&id), "ts": ts, "ct": b64u_encode(ct)});
    match peer_post(st, &node, "/n/deliver", &body).await {
        Ok(answer) => {
            let delivered = answer["delivered"].as_bool().unwrap_or(false);
            if !delivered {
                // gateway odběratele nemá: presence je zastaralá
                set_presence(st, *mailbox, &entry.gateway, 0);
            }
            delivered
        }
        Err(_) => false,
    }
}

/// Po nově uložené zprávě (na pozadí): doručit online, a když to nejde, poslat wake, má-li zpráva `push_id`.
pub fn after_store(st: &Arc<NodeState>, mailbox: [u8; 32], id: [u8; 16], ts: u64, ct: Vec<u8>, push_id: Option<String>) {
    let st = st.clone();
    tokio::spawn(async move {
        if deliver_online(&st, &mailbox, id, ts, &ct).await {
            return;
        }
        if let Some(push_id) = push_id {
            wake::spawn(&st, push_id, mailbox);
        }
    });
}
