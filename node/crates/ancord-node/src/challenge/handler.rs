use std::sync::Arc;
use std::time::Instant;

use reqwest::Method;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use super::cpu::compute_checkpoints;
use super::space::SpaceTable;
use crate::canonical::canonical_bytes;
use crate::encoding::{b64u_decode_array, b64u_encode};
use crate::sign::DOMAIN_REPORT;
use crate::state::NodeState;
use crate::time::unix_now;
use crate::worker::{signed_header, WorkerClient};

const MAX_CPU_ITERS: u64 = 1 << 33;
const MAX_LANES: u64 = 64;
const MAX_INDICES: usize = 256;
const MAX_BW_BYTES: u64 = 64 * 1024 * 1024;

type Reply = Result<Value, String>;

/// Zpracuje `challenge` a vrátí tělo `challenge_response` (bez `t` a `id`). Chyba se pošle jako `{"error": …}`.
pub async fn answer(state: &Arc<NodeState>, worker: &WorkerClient, msg: &Value) -> Value {
    let result = match msg["kind"].as_str() {
        Some("cpu") => cpu(msg).await,
        Some("space_commit") => space_commit(state, msg).await,
        Some("space_open") => space_open(state, msg).await,
        Some("bandwidth") => bandwidth(worker, msg).await,
        Some("witness") => witness(state, msg).await,
        other => Err(format!("unsupported challenge kind {other:?}")),
    };
    result.unwrap_or_else(|e| json!({"error": e}))
}

fn uint(msg: &Value, key: &str) -> Result<u64, String> {
    msg[key].as_u64().ok_or_else(|| format!("missing or invalid {key}"))
}

async fn cpu(msg: &Value) -> Reply {
    let seed = msg["seed"].as_str().and_then(b64u_decode_array::<32>).ok_or("invalid seed")?;
    let (iters, every, lanes) = (uint(msg, "iters")?, uint(msg, "every")?, uint(msg, "lanes")?);
    if iters > MAX_CPU_ITERS || lanes == 0 || lanes > MAX_LANES {
        return Err("cpu challenge exceeds local limits".into());
    }
    let cps = tokio::task::spawn_blocking(move || compute_checkpoints(&seed, lanes as u32, iters, every))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;
    Ok(json!({"checkpoints": cps}))
}

fn tier_of(msg: &Value) -> Result<&str, String> {
    match msg["tier"].as_str() {
        Some(t @ ("ram" | "storage")) => Ok(t),
        _ => Err("invalid tier".into()),
    }
}

async fn space_commit(state: &Arc<NodeState>, msg: &Value) -> Reply {
    let tier = tier_of(msg)?.to_owned();
    let seed = msg["seed"].as_str().and_then(b64u_decode_array::<32>).ok_or("invalid seed")?;
    let leaves = uint(msg, "leaves")?;
    if leaves > state.cfg.max_space_leaves {
        return Err(format!("leaves {leaves} exceeds local limit {}", state.cfg.max_space_leaves));
    }
    let dir = state.cfg.data_dir.clone();
    let tier_for_build = tier.clone();
    let started = Instant::now();
    let table = tokio::task::spawn_blocking(move || {
        let backing = (tier_for_build == "storage").then_some((dir.as_path(), "storage"));
        SpaceTable::build(seed, leaves, backing)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;
    let root = table.root();
    tracing::info!(%tier, leaves, ms = started.elapsed().as_millis() as u64, "space commitment built");
    // starý commitment se uvolní (u storage tím zmizí soubory) až po vytvoření nového
    let old = state.spaces.lock().map_err(|_| "state poisoned")?.insert(tier, Arc::new(table));
    drop(old);
    Ok(json!({"root": b64u_encode(&root)}))
}

async fn space_open(state: &Arc<NodeState>, msg: &Value) -> Reply {
    let tier = tier_of(msg)?;
    let table = state.spaces.lock().map_err(|_| "state poisoned")?.get(tier).cloned().ok_or("no commitment")?;
    let indices: Vec<Value> = msg["indices"].as_array().ok_or("missing indices")?.clone();
    if indices.len() > MAX_INDICES {
        return Err("too many indices".into());
    }
    let openings = tokio::task::spawn_blocking(move || {
        indices
            .iter()
            .map(|i| i.as_u64().ok_or_else(|| "invalid index".to_string()).and_then(|i| table.opening(i).map_err(|e| e.to_string())))
            .collect::<Result<Vec<_>, _>>()
    })
    .await
    .map_err(|e| e.to_string())??;
    Ok(json!({"openings": openings}))
}

/// URL výzvy se bere jen jako cesta vůči Workeru, který už známe: výzva nás nesmí poslat jinam.
fn worker_url_for(worker: &WorkerClient, url: &str) -> Result<String, String> {
    let rest = url.split_once("://").map(|(_, r)| r).ok_or("invalid url")?;
    let path = rest.find('/').map(|i| &rest[i..]).ok_or("invalid url")?;
    if !(path.starts_with("/challenge/blob/") || path.starts_with("/challenge/upload/")) {
        return Err("unexpected challenge url".into());
    }
    Ok(format!("{}{path}", worker.base()))
}

async fn bandwidth(worker: &WorkerClient, msg: &Value) -> Reply {
    let expected = uint(msg, "bytes")?;
    if expected == 0 || expected > MAX_BW_BYTES {
        return Err("bandwidth challenge exceeds local limits".into());
    }
    let dl = worker_url_for(worker, msg["dl_url"].as_str().ok_or("missing dl_url")?)?;
    let ul = worker_url_for(worker, msg["ul_url"].as_str().ok_or("missing ul_url")?)?;

    let res = worker.http().get(&dl).send().await.map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        return Err(format!("download failed: {}", res.status()));
    }
    let data = res.bytes().await.map_err(|e| e.to_string())?;
    if data.len() as u64 != expected {
        return Err(format!("download returned {} bytes, expected {expected}", data.len()));
    }
    let hash = b64u_encode(&Sha256::digest(&data));
    let up = worker.http().request(Method::PUT, &ul).body(data).send().await.map_err(|e| e.to_string())?;
    if !up.status().is_success() {
        return Err(format!("upload failed: {}", up.status()));
    }
    Ok(json!({"sha256": hash}))
}

/// Peer witness (§9.4): stáhne data z cizího nodu a podepíše report.
async fn witness(state: &Arc<NodeState>, msg: &Value) -> Reply {
    let target_id = msg["target_id"].as_str().ok_or("missing target_id")?;
    let host = msg["target_host"].as_str().ok_or("missing target_host")?;
    let bytes = uint(msg, "bytes")?;
    if bytes == 0 || bytes > MAX_BW_BYTES {
        return Err("witness challenge exceeds local limits".into());
    }
    // cíl musí být node z našeho adresáře a jeho hostname musí sedět
    let dir = state.directory().ok_or("no directory yet")?;
    if !dir.nodes.iter().any(|n| n.id == target_id && n.host == host) {
        return Err("target is not in the directory".into());
    }
    let path = format!("/.well-known/melo-bw?bytes={bytes}");
    let header = signed_header(state.identity.signing_key(), "GET", &path, &[]);
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(120)).build().map_err(|e| e.to_string())?;
    let started = Instant::now();
    let res = client.get(format!("https://{host}{path}")).header("authorization", header).send().await.map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        return Err(format!("target answered {}", res.status()));
    }
    let got = res.bytes().await.map_err(|e| e.to_string())?;
    if got.len() as u64 != bytes {
        return Err("target returned wrong size".into());
    }
    let duration_ms = (started.elapsed().as_millis() as u64).max(1);
    let report = json!({
        "target_id": target_id,
        "witness_id": state.identity.node_id(),
        "bytes": bytes,
        "duration_ms": duration_ms,
        "ts": unix_now(),
    });
    let payload = canonical_bytes(&report).map_err(|e| e.to_string())?;
    let sig = state.identity.sign(DOMAIN_REPORT, &payload);
    Ok(json!({"report": report, "sig": b64u_encode(&sig)}))
}
