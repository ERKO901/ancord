//! Řídicí WebSocket k Workeru: heartbeat, ping/pong, adresář a challenge (`spec/PROTOCOL.md` §5).

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::Message;

use crate::canonical::canonical_bytes;
use crate::challenge::handler;
use crate::encoding::b64u_encode;
use crate::sign::DOMAIN_HEARTBEAT;
use crate::state::NodeState;
use crate::ticket::verify_directory;
use crate::time::unix_now;
use crate::worker::{signed_header, WorkerClient};

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Spojení spadlo, stačí se připojit znovu.
    Reconnect,
    /// Worker nás nezná nebo nejsme `active`: zopakovat registraci.
    Reregister,
    /// Ban (kód 4003). Dál nemá smysl pokračovat.
    Banned,
}

const KEEPALIVE: Duration = Duration::from_secs(30);

fn heartbeat(state: &NodeState, seq: u64) -> Option<String> {
    use std::collections::BTreeMap;
    let (cpu_pct, mem_pct) = state.load.sample();
    let mut body: BTreeMap<&str, Value> = BTreeMap::new();
    body.insert("t", json!("hb"));
    body.insert("seq", json!(seq));
    body.insert("ts", json!(unix_now()));
    body.insert(
        "load",
        json!({
            "conns": state.conns.load(Ordering::Relaxed),
            "max_conns": state.cfg.max_conns,
            "cpu_pct": cpu_pct,
            "mem_pct": mem_pct,
        }),
    );
    if let Some(epoch) = state.epoch() {
        body.insert("epoch", json!(epoch));
    }
    let unsigned = serde_json::to_value(&body).ok()?;
    let sig = state.identity.sign(DOMAIN_HEARTBEAT, &canonical_bytes(&unsigned).ok()?);
    let mut msg = unsigned;
    msg["sig"] = json!(b64u_encode(&sig));
    serde_json::to_string(&msg).ok()
}

/// Jedno spojení po dobu jeho života. Vrací důvod ukončení.
pub async fn run(state: &Arc<NodeState>, worker: &WorkerClient) -> Outcome {
    let url = state.cfg.worker_ws_url();
    let mut request = match url.as_str().into_client_request() {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(error = %e, "invalid worker url");
            return Outcome::Reconnect;
        }
    };
    let header = signed_header(state.identity.signing_key(), "GET", "/ws", &[]);
    match header.parse() {
        Ok(v) => {
            request.headers_mut().insert("authorization", v);
        }
        Err(_) => return Outcome::Reconnect,
    }

    let ws = match tokio_tungstenite::connect_async(request).await {
        Ok((ws, _)) => ws,
        Err(tokio_tungstenite::tungstenite::Error::Http(res)) => {
            let status = res.status().as_u16();
            tracing::warn!(status, "websocket upgrade refused");
            return if status == 403 { Outcome::Reregister } else { Outcome::Reconnect };
        }
        Err(e) => {
            tracing::warn!(error = %e, "websocket connect failed");
            return Outcome::Reconnect;
        }
    };
    tracing::info!("control connection established");

    let (mut sink, mut stream) = ws.split();
    let (out_tx, mut out_rx) = mpsc::channel::<String>(32);
    let writer = tokio::spawn(async move {
        while let Some(text) = out_rx.recv().await {
            if sink.send(Message::text(text)).await.is_err() {
                break;
            }
        }
        let _ = sink.close().await;
    });

    let mut hb_interval = Duration::from_secs(15);
    let mut seq: u64 = 0;
    let mut next_hb = tokio::time::Instant::now();
    let mut keepalive = tokio::time::interval(KEEPALIVE);
    keepalive.tick().await;
    let mut outcome = Outcome::Reconnect;

    loop {
        tokio::select! {
            () = tokio::time::sleep_until(next_hb) => {
                if let Some(text) = heartbeat(state, seq) {
                    if out_tx.send(text).await.is_err() { break; }
                    seq += 1;
                }
                next_hb = tokio::time::Instant::now() + hb_interval;
            }
            _ = keepalive.tick() => {
                if out_tx.send("ping".to_owned()).await.is_err() { break; }
            }
            frame = stream.next() => {
                let Some(frame) = frame else { break };
                match frame {
                    Ok(Message::Text(text)) => {
                        if text.as_str() == "pong" { continue; }
                        let Ok(msg) = serde_json::from_str::<Value>(text.as_str()) else { continue };
                        match msg["t"].as_str() {
                            Some("welcome") => {
                                if let Some(s) = msg["heartbeat_interval_s"].as_u64().filter(|s| *s > 0) {
                                    hb_interval = Duration::from_secs(s);
                                }
                            }
                            Some("ack") => {}
                            Some("directory") => on_directory(state, &msg["directory"]),
                            Some("ping") => {
                                let pong = json!({"t": "pong", "id": msg["id"]}).to_string();
                                if out_tx.send(pong).await.is_err() { break; }
                            }
                            Some("challenge") => {
                                let (state, worker, tx) = (state.clone(), worker.clone(), out_tx.clone());
                                tokio::spawn(async move {
                                    let mut reply = handler::answer(&state, &worker, &msg).await;
                                    reply["t"] = json!("challenge_response");
                                    reply["id"] = msg["id"].clone();
                                    let _ = tx.send(reply.to_string()).await;
                                });
                            }
                            Some("bye") => tracing::info!(reason = %msg["reason"], "worker said bye"),
                            Some("error") => tracing::warn!(reason = %msg["reason"], "worker rejected a message"),
                            _ => {}
                        }
                    }
                    Ok(Message::Close(frame)) => {
                        if let Some(f) = frame {
                            let code = u16::from(f.code);
                            tracing::info!(code, reason = %f.reason, "control connection closed by worker");
                            if code == 4003 {
                                outcome = Outcome::Banned;
                            } else if f.code == CloseCode::Normal && f.reason.as_str() == "deregistered" {
                                outcome = Outcome::Reregister;
                            }
                        }
                        break;
                    }
                    Ok(_) => {}
                    Err(e) => {
                        tracing::warn!(error = %e, "control connection error");
                        break;
                    }
                }
            }
        }
    }
    drop(out_tx);
    writer.abort();
    outcome
}

fn on_directory(state: &NodeState, envelope: &Value) {
    match verify_directory(&state.worker_key, envelope) {
        Some(dir) => {
            let epoch = dir.epoch;
            if state.set_directory(dir) {
                tracing::info!(epoch, "directory updated");
            }
        }
        None => tracing::warn!("ignored directory with invalid signature"),
    }
}
