use std::io::IsTerminal;
use std::process::ExitCode;

use ancord_node::config::Config;
use ancord_node::encoding::{b64u_decode_array, b64u_encode};
use ancord_node::error::NodeError;
use ancord_node::identity::Identity;
use ancord_node::state::NodeState;
use ancord_node::worker::WorkerClient;
use ancord_node::{server, supervisor};
use clap::Parser;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_ansi(std::io::stderr().is_terminal())
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!("{e}");
            ExitCode::FAILURE
        }
    }
}

/// Klíč Workeru: zadaný v konfiguraci, jinak uložený z prvního startu, jinak stažený a uložený (TOFU).
async fn resolve_worker_key(cfg: &Config, worker: &WorkerClient) -> Result<[u8; 32], NodeError> {
    if let Some(configured) = &cfg.worker_pubkey {
        return b64u_decode_array(configured).ok_or_else(|| NodeError("WORKER_PUBKEY is not a 32 byte base64url key".into()));
    }
    let path = cfg.data_dir.join("worker.pub");
    if let Ok(text) = std::fs::read_to_string(&path) {
        return b64u_decode_array(text.trim()).ok_or_else(|| NodeError(format!("{} is corrupt", path.display())));
    }
    let key = worker.worker_key().await?;
    std::fs::write(&path, format!("{}\n", b64u_encode(&key)))?;
    tracing::warn!("worker key pinned on first use; set WORKER_PUBKEY to pin it explicitly");
    Ok(key)
}

async fn run() -> Result<(), NodeError> {
    let cfg = Config::parse();
    let identity = Identity::load_or_create(&cfg.data_dir)?;
    tracing::info!(node_id = identity.node_id(), pubkey = %identity.pubkey_b64(), "identity loaded");
    let worker = WorkerClient::new(&cfg.worker_url)?;
    let worker_key = resolve_worker_key(&cfg, &worker).await?;
    let state = NodeState::new(cfg.clone(), identity, worker_key);

    let listener = tokio::net::TcpListener::bind(cfg.listen).await?;
    tracing::info!(listen = %cfg.listen, node = %cfg.node_name, "http server up");
    let app = server::router(state.clone());
    let server = tokio::spawn(async move { axum::serve(listener, app).await });

    tokio::select! {
        r = supervisor::run(state, worker) => r,
        r = server => Err(NodeError(format!("http server stopped: {r:?}"))),
        _ = tokio::signal::ctrl_c() => { tracing::info!("shutting down"); Ok(()) }
    }
}
