//! Klient HTTP API Workeru (`spec/PROTOCOL.md` §3 a §4).

use std::time::Duration;

use ed25519_dalek::SigningKey;
use rand::RngCore;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::config::Claimed;
use crate::encoding::{b64u_decode_array, b64u_encode};
use crate::error::WorkerError;
use crate::sign::auth_header;
use crate::time::unix_now;

#[derive(Debug, Deserialize)]
pub struct BeginResponse {
    pub node_id: String,
    pub hostname: String,
    pub challenge_nonce: String,
    pub hashcash_bits: u32,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmResponse {
    pub node_id: String,
    pub hostname: String,
    pub status: String,
    pub tunnel_token: String,
    pub activate_before: u64,
}

#[derive(Clone)]
pub struct WorkerClient {
    http: reqwest::Client,
    base: String,
}

impl WorkerClient {
    pub fn new(base: &str) -> Result<Self, WorkerError> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(concat!("ancord-node/", env!("CARGO_PKG_VERSION")))
            .build()?;
        Ok(Self { http, base: base.trim_end_matches('/').to_owned() })
    }

    pub fn base(&self) -> &str {
        &self.base
    }

    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }

    async fn finish(res: reqwest::Response) -> Result<Value, WorkerError> {
        let status = res.status();
        let retry_after = res.headers().get("retry-after").and_then(|v| v.to_str().ok()).and_then(|v| v.parse().ok());
        let text = res.text().await?;
        if status.is_success() {
            return serde_json::from_str(&text).map_err(|e| WorkerError::Protocol(format!("invalid JSON from worker: {e}")));
        }
        let err: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
        let field = |k: &str| err["error"][k].as_str().unwrap_or("").to_owned();
        Err(WorkerError::Api { status: status.as_u16(), code: field("code"), message: field("message"), retry_after })
    }

    fn decode<T: for<'de> Deserialize<'de>>(v: Value) -> Result<T, WorkerError> {
        serde_json::from_value(v).map_err(|e| WorkerError::Protocol(format!("unexpected response shape: {e}")))
    }

    /// `GET /.well-known/ancord-key`.
    pub async fn worker_key(&self) -> Result<[u8; 32], WorkerError> {
        let v = Self::finish(self.http.get(format!("{}/.well-known/ancord-key", self.base)).send().await?).await?;
        v["pubkey"]
            .as_str()
            .and_then(b64u_decode_array)
            .ok_or_else(|| WorkerError::Protocol("worker key is not 32 bytes".into()))
    }

    pub async fn register_begin(&self, pubkey: &str, claimed: &Claimed) -> Result<BeginResponse, WorkerError> {
        let body = json!({"pubkey": pubkey, "claimed_specs": claimed});
        let res = self.http.post(format!("{}/register", self.base)).json(&body).send().await?;
        Self::decode(Self::finish(res).await?)
    }

    pub async fn register_confirm(
        &self,
        pubkey: &str,
        nonce: &str,
        signature: &str,
        hashcash_counter: Option<u64>,
    ) -> Result<ConfirmResponse, WorkerError> {
        let mut body = json!({"pubkey": pubkey, "challenge_nonce": nonce, "signature": signature});
        if let Some(c) = hashcash_counter {
            body["hashcash_counter"] = json!(c);
        }
        let res = self.http.post(format!("{}/register/confirm", self.base)).json(&body).send().await?;
        Self::decode(Self::finish(res).await?)
    }

    /// Podepsaný request (§3). Tělo se serializuje jednou, podpis je přes přesně ty bajty, které se odešlou.
    pub async fn signed(&self, key: &SigningKey, method: reqwest::Method, path: &str, body: Option<&Value>) -> Result<Value, WorkerError> {
        let bytes = match body {
            Some(v) => serde_json::to_vec(v).map_err(|e| WorkerError::Protocol(e.to_string()))?,
            None => Vec::new(),
        };
        let header = signed_header(key, method.as_str(), path, &bytes);
        let mut req = self.http.request(method, format!("{}{path}", self.base)).header("authorization", header);
        if body.is_some() {
            req = req.header("content-type", "application/json").body(bytes);
        }
        Self::finish(req.send().await?).await
    }
}

pub fn signed_header(key: &SigningKey, method: &str, path_and_query: &str, body: &[u8]) -> String {
    let mut nonce = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut nonce);
    auth_header(key, method, path_and_query, body, unix_now(), &nonce)
}

/// Základ pro exponenciální čekání s horní mezí; `retry_after` od Workeru má přednost.
pub fn backoff(attempt: u32, retry_after: Option<u64>) -> Duration {
    if let Some(s) = retry_after {
        return Duration::from_secs(s.clamp(1, 300));
    }
    Duration::from_millis((500u64 << attempt.min(6)).min(30_000))
}

pub fn nonce_b64() -> String {
    let mut n = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut n);
    b64u_encode(&n)
}
