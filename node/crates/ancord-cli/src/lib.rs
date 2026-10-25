//! Minimální klient: `GET /route`, připojení na node s ticketem a failover (`spec/PROTOCOL.md` §8 a §8.1).

use std::time::Duration;

use ancord_node::encoding::{b64u_decode_array, b64u_encode};
use ancord_node::sign::{sign, DOMAIN_CLIENT_AUTH};
use ed25519_dalek::SigningKey;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub type NodeSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("route: {0}")]
    Route(String),
    #[error("{node}: {reason}")]
    Admission { node: String, reason: String },
    #[error("no node accepted the connection: {0}")]
    AllFailed(String),
}

#[derive(Debug, Clone, Deserialize)]
pub struct RouteNode {
    pub node_id: String,
    pub hostname: String,
    pub ticket: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Route {
    pub epoch: u64,
    pub ttl: u64,
    pub nodes: Vec<RouteNode>,
}

/// `GET /route?ek=…` (spec §8).
pub async fn fetch_route(http: &reqwest::Client, worker: &str, ek_pub: &[u8; 32]) -> Result<Route, CliError> {
    let url = format!("{}/route?ek={}", worker.trim_end_matches('/'), b64u_encode(ek_pub));
    let res = http.get(url).send().await.map_err(|e| CliError::Route(e.to_string()))?;
    let status = res.status();
    let text = res.text().await.map_err(|e| CliError::Route(e.to_string()))?;
    if !status.is_success() {
        return Err(CliError::Route(format!("{status}: {text}")));
    }
    serde_json::from_str(&text).map_err(|e| CliError::Route(format!("unexpected response: {e}")))
}

fn admission(node: &str, reason: impl Into<String>) -> CliError {
    CliError::Admission { node: node.to_owned(), reason: reason.into() }
}

async fn next_json(ws: &mut NodeSocket, node: &str, wait: Duration) -> Result<Value, CliError> {
    loop {
        let frame = tokio::time::timeout(wait, ws.next())
            .await
            .map_err(|_| admission(node, "timeout"))?
            .ok_or_else(|| admission(node, "connection closed"))?
            .map_err(|e| admission(node, e.to_string()))?;
        match frame {
            Message::Text(t) => return serde_json::from_str(t.as_str()).map_err(|e| admission(node, e.to_string())),
            Message::Close(f) => {
                let why = f.map_or_else(|| "closed".to_owned(), |f| format!("closed {} {}", u16::from(f.code), f.reason));
                return Err(admission(node, why));
            }
            _ => {}
        }
    }
}

/// Admission na jednom nodu: přijmout `hello`, odeslat `auth`, počkat na `ready` (spec §8.1).
pub async fn admit(url: &str, node: &RouteNode, ek: &SigningKey) -> Result<NodeSocket, CliError> {
    let wait = Duration::from_secs(15);
    let (mut ws, _) = tokio_tungstenite::connect_async(url).await.map_err(|e| admission(&node.node_id, e.to_string()))?;
    let hello = next_json(&mut ws, &node.node_id, wait).await?;
    if hello["t"] != "hello" || hello["node_id"] != node.node_id.as_str() {
        return Err(admission(&node.node_id, "unexpected hello"));
    }
    let challenge = hello["challenge"].as_str().and_then(b64u_decode_array::<32>).ok_or_else(|| admission(&node.node_id, "bad challenge"))?;
    let mut payload = challenge.to_vec();
    payload.extend_from_slice(node.node_id.as_bytes());
    let sig = sign(ek, DOMAIN_CLIENT_AUTH, &payload);
    let auth = json!({"t": "auth", "ticket": node.ticket, "sig": b64u_encode(&sig)});
    ws.send(Message::text(auth.to_string())).await.map_err(|e| admission(&node.node_id, e.to_string()))?;
    let answer = next_json(&mut ws, &node.node_id, wait).await?;
    match answer["t"].as_str() {
        Some("ready") => Ok(ws),
        _ => Err(admission(&node.node_id, answer["reason"].as_str().unwrap_or("rejected"))),
    }
}

/// Zkouší nody v pořadí z `/route`, první, který klienta přijme, vyhrává (failover).
/// `url_for` převádí hostname na adresu WebSocketu (výchozí `wss://<hostname>/client`).
pub async fn connect_any(route: &Route, ek: &SigningKey, url_for: impl Fn(&str) -> String) -> Result<(RouteNode, NodeSocket), CliError> {
    let mut failures = Vec::new();
    for node in &route.nodes {
        match admit(&url_for(&node.hostname), node, ek).await {
            Ok(ws) => return Ok((node.clone(), ws)),
            Err(e) => failures.push(e.to_string()),
        }
    }
    Err(CliError::AllFailed(failures.join("; ")))
}

pub fn default_url(hostname: &str) -> String {
    format!("wss://{hostname}/client")
}
