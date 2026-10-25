//! Životní cyklus nodu: registrace, tunel, řídicí spojení a opakování po výpadku.

use std::sync::Arc;
use std::time::Duration;

use crate::control::{self, Outcome};
use crate::error::NodeError;
use crate::register;
use crate::state::NodeState;
use crate::tunnel::Tunnel;
use crate::worker::{backoff, WorkerClient};

/// Běží, dokud node neukončí ban nebo signál. `Ok` znamená čisté ukončení.
pub async fn run(state: Arc<NodeState>, worker: WorkerClient) -> Result<(), NodeError> {
    let measured = crate::sysinfo::measure(&state.cfg.data_dir);
    let claimed = state.cfg.claimed(&measured);
    tracing::info!(node_id = state.identity.node_id(), ?claimed, "starting");

    let mut tunnel: Option<Tunnel> = None;
    let mut attempt = 0u32;
    loop {
        // 1. registrace (idempotentní: po restartu vrátí znovu token)
        let reg = match register::register(&worker, &state.identity, &claimed).await {
            Ok(r) => r,
            Err(e) if e.is_retryable() => {
                tracing::warn!(error = %e, "registration failed, will retry");
                tokio::time::sleep(backoff(attempt, e.retry_after())).await;
                attempt += 1;
                continue;
            }
            Err(e) => return Err(e.into()),
        };
        state.set_hostname(&reg.hostname);

        // 2. tunel (ne v lokálním vývoji)
        if state.cfg.cloudflared_bin != "none" {
            tunnel = Some(Tunnel::spawn(state.cfg.cloudflared_bin.clone(), reg.tunnel_token.clone()));
        } else if tunnel.is_none() {
            tracing::warn!("tunnel disabled (CLOUDFLARED_BIN=none)");
        }

        // 3. aktivace
        if let Err(e) = register::activate(&worker, &state.identity, reg.activate_before).await {
            tracing::warn!(error = %e, "activation failed, registering again");
            tokio::time::sleep(backoff(attempt, e.retry_after())).await;
            attempt += 1;
            continue;
        }
        attempt = 0;

        // 4. řídicí spojení; po jeho pádu se připojujeme znovu
        let mut reconnects = 0u32;
        loop {
            match control::run(&state, &worker).await {
                Outcome::Banned => return Err(NodeError("node is banned by the worker".into())),
                Outcome::Reregister => break,
                Outcome::Reconnect => {
                    tokio::time::sleep(backoff(reconnects, None).min(Duration::from_secs(15))).await;
                    reconnects = reconnects.saturating_add(1);
                }
            }
        }
    }
}
