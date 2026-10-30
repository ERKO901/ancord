//! Push wake: anchor, který nově uložil zprávu, upozorní Worker (`spec/PROTOCOL.md` §10.2 a §13.6).

use std::sync::Arc;

use reqwest::Method;
use serde_json::json;

use crate::encoding::b64u_encode;
use crate::state::NodeState;
use crate::time::unix_now;

pub use ancord_core::wake::*;

/// Lokální koalescence: `true`, když se pro `push_id` smí wake poslat teď.
fn admit(st: &NodeState, push_id: &str, now: u64) -> bool {
    let Ok(mut last) = st.wake_last.lock() else {
        return false;
    };
    let interval = st.cfg.wake_min_interval_s;
    last.retain(|_, at| at.saturating_add(interval) > now);
    if last.contains_key(push_id) {
        return false;
    }
    last.insert(push_id.to_owned(), now);
    true
}

/// Pošle wake na pozadí. Selhání se jen zaloguje, zápis zprávy se tím nikdy neruší.
pub fn spawn(st: &Arc<NodeState>, push_id: String, mailbox_id: [u8; 32]) {
    if !admit(st, &push_id, unix_now()) {
        return;
    }
    let st = st.clone();
    tokio::spawn(async move {
        let body = json!({"push_id": push_id, "mailbox_id": b64u_encode(&mailbox_id)});
        match st.worker.signed(st.identity.signing_key(), Method::POST, "/wake", Some(&body)).await {
            Ok(answer) => {
                let status = answer["status"].as_str().unwrap_or("?");
                // žádný push neodešel (příjemce token zatím neregistroval): příští zpráva má smět budit hned
                if status == "no_token" {
                    if let Ok(mut last) = st.wake_last.lock() {
                        last.remove(&push_id);
                    }
                }
                tracing::debug!(status, "wake sent");
            }
            Err(e) => tracing::warn!(error = %e, "wake failed"),
        }
    });
}
