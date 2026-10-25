//! Registrace nodu: `register → confirm → activate` (`spec/PROTOCOL.md` §4).

use std::time::Duration;

use reqwest::Method;
use serde_json::json;

use crate::config::Claimed;
use crate::encoding::{b64u_decode_array, b64u_encode};
use crate::error::WorkerError;
use crate::hashcash;
use crate::identity::{hostname_matches, Identity};
use crate::sign::DOMAIN_REGISTER;
use crate::time::unix_now;
use crate::worker::{backoff, WorkerClient};

pub struct Registration {
    pub hostname: String,
    pub tunnel_token: String,
    pub activate_before: u64,
}

/// Kroky 1 a 2. Opakovaná registrace téhož klíče (restart) vrátí znovu token bez nových zdrojů.
pub async fn register(worker: &WorkerClient, id: &Identity, claimed: &Claimed) -> Result<Registration, WorkerError> {
    let pubkey = id.pubkey_b64();
    let begin = worker.register_begin(&pubkey, claimed).await?;
    if begin.node_id != id.node_id() || !hostname_matches(id.node_id(), &begin.hostname) {
        return Err(WorkerError::Protocol("worker returned a node_id or hostname that does not match our key".into()));
    }
    let nonce: [u8; 32] =
        b64u_decode_array(&begin.challenge_nonce).ok_or_else(|| WorkerError::Protocol("challenge_nonce is not 32 bytes".into()))?;

    let mut payload = nonce.to_vec();
    payload.extend_from_slice(begin.hostname.as_bytes());
    payload.extend_from_slice(&id.pubkey());
    let signature = b64u_encode(&id.sign(DOMAIN_REGISTER, &payload));

    let counter = if begin.hashcash_bits > 0 {
        let pk = id.pubkey();
        let bits = begin.hashcash_bits;
        let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
        tracing::info!(bits, "solving hashcash");
        let c = tokio::task::spawn_blocking(move || hashcash::solve(&nonce, &pk, bits, threads))
            .await
            .map_err(|e| WorkerError::Protocol(format!("hashcash task failed: {e}")))?;
        Some(c)
    } else {
        None
    };

    let confirm = worker.register_confirm(&pubkey, &begin.challenge_nonce, &signature, counter).await?;
    if confirm.node_id != id.node_id() || confirm.hostname != begin.hostname {
        return Err(WorkerError::Protocol("confirm response does not match registration".into()));
    }
    tracing::info!(status = %confirm.status, hostname = %confirm.hostname, "registered");
    Ok(Registration { hostname: confirm.hostname, tunnel_token: confirm.tunnel_token, activate_before: confirm.activate_before })
}

/// Krok 3: Worker zavolá náš `/.well-known/melo-proof`. Tunel se může rozběhnout se zpožděním, proto se 502 opakuje.
pub async fn activate(worker: &WorkerClient, id: &Identity, activate_before: u64) -> Result<(), WorkerError> {
    let mut attempt = 0u32;
    loop {
        match worker.signed(id.signing_key(), Method::POST, "/register/activate", Some(&json!({}))).await {
            Ok(_) => {
                tracing::info!("node activated");
                return Ok(());
            }
            Err(e) => {
                let late = activate_before != 0 && unix_now() >= activate_before;
                let retry = e.is_retryable() || matches!(e.code(), Some("upstream" | "proof_failed"));
                if late || !retry || attempt >= 20 {
                    return Err(e);
                }
                tracing::info!(error = %e, attempt, "activation not ready, retrying");
                tokio::time::sleep(backoff(attempt, e.retry_after()).min(Duration::from_secs(10))).await;
                attempt += 1;
            }
        }
    }
}
