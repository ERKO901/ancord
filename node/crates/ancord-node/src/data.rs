//! Data plane na straně nodu: anchory mailboxu, replikace s kvorem a handoff (`spec/PROTOCOL.md` §13).
//! Node vidí jen ciphertext; nikdy nezná odesílatele ani příjemce.

use std::collections::BTreeMap;
use std::future::Future;
use std::sync::Arc;

use serde_json::{json, Value};
use tokio::sync::mpsc;

use crate::encoding::{b64u_decode, b64u_decode_array, b64u_encode};
use crate::hrw;
use crate::n2n;
use crate::state::NodeState;
use crate::store::{MsgState, PutOutcome, Record, StoreError};
use crate::ticket::{DirectoryBody, DirectoryNode};
use crate::time::unix_now;
use crate::presence;

pub const MAX_CT_BYTES: usize = 32 * 1024;
pub const MAX_ACK_IDS: usize = 500;
pub const DEFAULT_FETCH: usize = 100;
pub const MAX_FETCH: usize = 500;
/// Jak moc se smí `ts` klienta lišit od času nodu.
const TS_SKEW_S: u64 = 86_400;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DataError {
    #[error("{0}")]
    BadRequest(&'static str),
    #[error("message is too large")]
    TooLarge,
    #[error("node has no directory yet")]
    NoDirectory,
    #[error("not enough nodes in the directory")]
    InsufficientNodes,
    #[error("anchors did not answer in sufficient numbers")]
    QuorumUnavailable,
    #[error("mailbox is full")]
    MailboxFull,
    #[error("not found")]
    NotFound,
    #[error("invalid signature or proof of work")]
    InvalidProof,
    /// Střežená schránka (§15.13): zápis potřebuje razítko s touto obtížností.
    #[error("inbox requires a proof-of-work stamp")]
    PowRequired(u8),
    #[error("storage failure")]
    Storage,
}

impl From<ancord_core::error::ProofError> for DataError {
    fn from(e: ancord_core::error::ProofError) -> Self {
        match e {
            ancord_core::error::ProofError::BadRequest(m) => Self::BadRequest(m),
            ancord_core::error::ProofError::InvalidProof => Self::InvalidProof,
        }
    }
}

impl DataError {
    /// Stabilní kód z `spec/PROTOCOL.md` §13.3.
    pub fn code(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "bad_request",
            Self::TooLarge => "too_large",
            Self::NoDirectory => "no_directory",
            Self::InsufficientNodes => "insufficient_nodes",
            Self::QuorumUnavailable => "quorum_unavailable",
            Self::MailboxFull => "mailbox_full",
            Self::NotFound => "not_found",
            Self::InvalidProof => "invalid_proof",
            Self::PowRequired(_) => "pow_required",
            Self::Storage => "unavailable",
        }
    }
}

impl From<StoreError> for DataError {
    fn from(e: StoreError) -> Self {
        match e {
            StoreError::Full => Self::MailboxFull,
            StoreError::Io(msg) => {
                tracing::error!(error = %msg, "mailbox storage failed");
                Self::Storage
            }
        }
    }
}

/// Zpráva v podobě, kterou anchor vrací.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stored {
    pub id: [u8; 16],
    pub ts: u64,
    pub exp: u64,
    pub ct: Vec<u8>,
}

impl Stored {
    pub fn to_json(&self, with_exp: bool) -> Value {
        let mut v = json!({"id": b64u_encode(&self.id), "ts": self.ts, "ct": b64u_encode(&self.ct)});
        if with_exp {
            v["exp"] = json!(self.exp);
        }
        v
    }

    fn from_json(v: &Value) -> Option<Self> {
        Some(Self {
            id: b64u_decode_array(v["id"].as_str()?)?,
            ts: v["ts"].as_u64()?,
            exp: v["exp"].as_u64()?,
            ct: b64u_decode(v["ct"].as_str()?)?,
        })
    }
}

pub fn parse_mailbox(s: &str) -> Result<[u8; 32], DataError> {
    b64u_decode_array(s).ok_or(DataError::BadRequest("invalid mailbox_id"))
}

pub fn parse_id(s: &str) -> Result<[u8; 16], DataError> {
    b64u_decode_array(s).ok_or(DataError::BadRequest("invalid message id"))
}

pub fn parse_ct(s: &str) -> Result<Vec<u8>, DataError> {
    if s.len() > MAX_CT_BYTES * 4 / 3 + 4 {
        return Err(DataError::TooLarge);
    }
    let ct = b64u_decode(s).ok_or(DataError::BadRequest("invalid ct"))?;
    if ct.is_empty() {
        return Err(DataError::BadRequest("empty ct"));
    }
    if ct.len() > MAX_CT_BYTES {
        return Err(DataError::TooLarge);
    }
    Ok(ct)
}

pub fn check_ts(ts: u64, now: u64) -> Result<(), DataError> {
    if now.abs_diff(ts) > TS_SKEW_S {
        return Err(DataError::BadRequest("ts is out of range"));
    }
    Ok(())
}

pub fn clamp_limit(limit: Option<u64>) -> usize {
    limit.map_or(DEFAULT_FETCH, |l| usize::try_from(l).unwrap_or(MAX_FETCH).clamp(1, MAX_FETCH))
}

/// Anchory mailboxu v aktuální epoše a z nich odvozené kvorum.
pub(crate) struct Plan {
    pub(crate) nodes: Vec<DirectoryNode>,
    pub(crate) quorum: usize,
}

impl Plan {
    /// Počet odpovědí, jejichž sjednocení se vždy protne s jakýmkoli zápisem na kvorum.
    pub(crate) fn read_quorum(&self) -> usize {
        (self.nodes.len() + 1).saturating_sub(self.quorum).max(1)
    }
}

pub(crate) fn anchors_in(dir: &DirectoryBody, mailbox_id: &[u8; 32]) -> Vec<DirectoryNode> {
    let r = usize::try_from(dir.params.r).unwrap_or(usize::MAX);
    let ids = hrw::anchors(mailbox_id, dir.nodes.iter().map(|n| n.id.as_str()), dir.epoch, r);
    ids.iter().filter_map(|id| dir.nodes.iter().find(|n| &n.id == id).cloned()).collect()
}

pub(crate) fn plan(st: &NodeState, mailbox_id: &[u8; 32]) -> Result<Plan, DataError> {
    let dir = st.directory().ok_or(DataError::NoDirectory)?;
    let quorum = usize::try_from(dir.params.quorum).unwrap_or(usize::MAX);
    if quorum == 0 || dir.nodes.len() < quorum {
        return Err(DataError::InsufficientNodes);
    }
    Ok(Plan { nodes: anchors_in(&dir, mailbox_id), quorum })
}

/// Je tento node anchorem mailboxu v aktuální nebo předchozí epoše?
pub fn is_anchor(st: &NodeState, mailbox_id: &[u8; 32]) -> bool {
    let me = st.identity.node_id();
    [st.directory(), st.prev_directory()].into_iter().flatten().any(|d| anchors_in(&d, mailbox_id).iter().any(|n| n.id == me))
}

/// Node z aktuálního nebo předchozího adresáře podle jeho `node_id`.
pub fn find_node(st: &NodeState, node_id: &str) -> Option<DirectoryNode> {
    [st.directory(), st.prev_directory()].into_iter().flatten().find_map(|d| d.nodes.iter().find(|n| n.id == node_id).cloned())
}

/// Je `pubkey` anchorem mailboxu v aktuální nebo předchozí epoše?
pub fn is_anchor_pubkey(st: &NodeState, pubkey: &[u8; 32], mailbox_id: &[u8; 32]) -> bool {
    [st.directory(), st.prev_directory()]
        .into_iter()
        .flatten()
        .any(|d| anchors_in(&d, mailbox_id).iter().any(|n| b64u_decode_array::<32>(&n.pk).is_some_and(|pk| &pk == pubkey)))
}

/// Node z adresáře podle veřejného klíče.
pub fn node_by_pubkey(st: &NodeState, pubkey: &[u8; 32]) -> Option<DirectoryNode> {
    [st.directory(), st.prev_directory()]
        .into_iter()
        .flatten()
        .find_map(|d| d.nodes.iter().find(|n| b64u_decode_array::<32>(&n.pk).is_some_and(|pk| &pk == pubkey)).cloned())
}

/// Je `pubkey` nodem z aktuálního nebo předchozího adresáře?
pub fn is_member(st: &NodeState, pubkey: &[u8; 32]) -> bool {
    [st.directory(), st.prev_directory()]
        .into_iter()
        .flatten()
        .any(|d| d.nodes.iter().any(|n| b64u_decode_array::<32>(&n.pk).is_some_and(|pk| &pk == pubkey)))
}

pub(crate) async fn blocking<T, F>(st: &Arc<NodeState>, f: F) -> Result<T, DataError>
where
    F: FnOnce(&NodeState) -> Result<T, StoreError> + Send + 'static,
    T: Send + 'static,
{
    let st = st.clone();
    match tokio::task::spawn_blocking(move || f(&st)).await {
        Ok(r) => r.map_err(DataError::from),
        Err(_) => Err(DataError::Storage),
    }
}

/// Zápis do vlastního úložiště. `ct = None` zapíše náhrobek. `exp` se omezí na `now + message_ttl_s`.
pub async fn put_local(st: &Arc<NodeState>, mailbox_id: [u8; 32], id: [u8; 16], ts: u64, exp: u64, ct: Option<Vec<u8>>) -> Result<PutOutcome, DataError> {
    let now = unix_now();
    let exp = exp.min(now.saturating_add(st.cfg.message_ttl_s));
    if exp <= now {
        return Ok(PutOutcome::Duplicate);
    }
    // střežená schránka: razítko je součástí `ct`, takže platí i při handoffu (§15.13); náhrobek ho nepotřebuje
    if let Some(ct) = &ct {
        let bits = crate::guard::required_bits(st, mailbox_id).await?;
        if bits > 0 && !crate::guard::stamp_valid(&mailbox_id, &id, ct, bits) {
            return Err(DataError::PowRequired(bits));
        }
    }
    let cap = st.cfg.max_mailbox_messages;
    blocking(st, move |s| s.store.put(&mailbox_id, &id, ts, exp, ct.as_deref(), cap)).await
}

pub async fn get_local(st: &Arc<NodeState>, mailbox_id: [u8; 32], limit: usize) -> Result<Vec<Stored>, DataError> {
    let now = unix_now();
    let records = blocking(st, move |s| s.store.pending(&mailbox_id, limit, now)).await?;
    Ok(records.into_iter().filter_map(|r| Some(Stored { id: r.id, ts: r.ts, exp: r.exp, ct: r.ct? })).collect())
}

pub async fn ack_local(st: &Arc<NodeState>, mailbox_id: [u8; 32], ids: Vec<[u8; 16]>) -> Result<usize, DataError> {
    let now = unix_now();
    let exp = now.saturating_add(st.cfg.message_ttl_s);
    blocking(st, move |s| s.store.ack(&mailbox_id, &ids, now, exp)).await
}

pub async fn status_local(st: &Arc<NodeState>, mailbox_id: [u8; 32], ids: Vec<[u8; 16]>) -> Result<Vec<MsgState>, DataError> {
    let now = unix_now();
    blocking(st, move |s| s.store.states(&mailbox_id, &ids, now)).await
}

/// Stav zpráv podle odpovědí nejméně `r − quorum + 1` anchorů; „doručeno“ vyhrává nad „čeká“ a to nad „neznámé“.
pub async fn status(st: &Arc<NodeState>, mailbox_id: [u8; 32], ids: Vec<[u8; 16]>) -> Result<Vec<MsgState>, DataError> {
    let plan = plan(st, &mailbox_id)?;
    let need = plan.read_quorum();
    let count = ids.len();
    let ids = Arc::new(ids);
    let mut rx = fan_out(st, plan.nodes, move |st, node, is_self| {
        let ids = ids.clone();
        async move {
            if is_self {
                return status_local(&st, mailbox_id, ids.to_vec()).await.map_err(local);
            }
            let list: Vec<String> = ids.iter().map(|i| b64u_encode(i)).collect();
            let answer = peer_post(&st, &node, "/n/status", &json!({"mailbox_id": b64u_encode(&mailbox_id), "ids": list})).await?;
            let states: Option<Vec<MsgState>> =
                answer["states"].as_array().and_then(|a| a.iter().map(|v| v.as_str().and_then(MsgState::parse)).collect());
            states.filter(|s| s.len() == ids.len()).ok_or_else(|| OpError::Failed("malformed states".into()))
        }
    });
    let (mut answers, mut merged) = (0, vec![MsgState::Unknown; count]);
    while let Some(result) = rx.recv().await {
        if let Ok(list) = result {
            answers += 1;
            for (m, s) in merged.iter_mut().zip(list) {
                *m = (*m).max(s);
            }
            if answers >= need {
                return Ok(merged);
            }
        }
    }
    Err(DataError::QuorumUnavailable)
}

#[derive(Debug)]
pub(crate) enum OpError {
    Full,
    /// Anchor chce razítko s touto obtížností (§15.13).
    PowRequired(u8),
    Failed(String),
}

/// Kolikrát a jak dlouho se opakuje `403`: příjemce může být o pár vteřin pozadu se změnou epochy.
const FORBIDDEN_RETRIES: u32 = 2;
const FORBIDDEN_DELAY: std::time::Duration = std::time::Duration::from_millis(400);

/// Operace na jiném nodu přes zabezpečený kanál (`spec/PROTOCOL.md` §14.2). `path` je `/n/<op>`.
pub(crate) async fn peer_post(st: &NodeState, node: &DirectoryNode, path: &str, body: &Value) -> Result<Value, OpError> {
    let op = path.strip_prefix("/n/").unwrap_or(path);
    let mut attempt = 0;
    loop {
        let (status, answer) = n2n::call(st, node, op, body).await.map_err(OpError::Failed)?;
        if status == 403 && attempt < FORBIDDEN_RETRIES {
            attempt += 1;
            // příjemce může mít novější adresář, který jsme nedostali (Worker nedostupný, spec §6.1); s ním se mění
            // anchory, takže opakovat tento požadavek nemá smysl, další už půjde podle nového adresáře
            if attempt == 1 && crate::reach::pull_directory(st, node).await {
                return Err(OpError::Failed(format!("{} holds a newer directory", node.id)));
            }
            tokio::time::sleep(FORBIDDEN_DELAY).await;
            continue;
        }
        if status == 507 {
            return Err(OpError::Full);
        }
        if status == 428 {
            let bits = answer["error"]["bits"].as_u64().and_then(|b| u8::try_from(b).ok()).unwrap_or(crate::guard::MAX_BITS);
            return Err(OpError::PowRequired(bits));
        }
        if !(200..300).contains(&status) {
            return Err(OpError::Failed(format!("{} answered {status}", node.id)));
        }
        return Ok(answer);
    }
}

/// Spustí `op` na všech uzlech současně. Úlohy běží dál, i když volající přestane čekat.
pub(crate) fn fan_out<T, F, Fut>(st: &Arc<NodeState>, nodes: Vec<DirectoryNode>, op: F) -> mpsc::Receiver<Result<T, OpError>>
where
    T: Send + 'static,
    F: Fn(Arc<NodeState>, DirectoryNode, bool) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<T, OpError>> + Send + 'static,
{
    let (tx, rx) = mpsc::channel(nodes.len().max(1));
    let op = Arc::new(op);
    for node in nodes {
        let (st, tx, op) = (st.clone(), tx.clone(), op.clone());
        tokio::spawn(async move {
            let is_self = node.id == st.identity.node_id();
            let _ = tx.send(op(st, node, is_self).await).await;
        });
    }
    rx
}

pub(crate) fn local(e: DataError) -> OpError {
    match e {
        DataError::MailboxFull => OpError::Full,
        DataError::PowRequired(bits) => OpError::PowRequired(bits),
        other => OpError::Failed(other.to_string()),
    }
}

/// Zápis zprávy na všechny anchory; hotovo po `quorum` potvrzeních. Vrací počet potvrzení.
pub async fn send(st: &Arc<NodeState>, mailbox_id: [u8; 32], id: [u8; 16], ts: u64, ct: Vec<u8>) -> Result<usize, DataError> {
    send_with_push(st, mailbox_id, id, ts, ct, None).await
}

/// Jako `send`; s `push_id` příjemce anchory po uložení zprávy pošlou wake (`spec/PROTOCOL.md` §13.6).
pub async fn send_with_push(st: &Arc<NodeState>, mailbox_id: [u8; 32], id: [u8; 16], ts: u64, ct: Vec<u8>, push_id: Option<String>) -> Result<usize, DataError> {
    let plan = plan(st, &mailbox_id)?;
    let exp = unix_now().saturating_add(st.cfg.message_ttl_s);
    let ct = Arc::new(ct);
    let mut rx = fan_out(st, plan.nodes, move |st, node, is_self| {
        let (ct, push_id) = (ct.clone(), push_id.clone());
        async move {
            if is_self {
                let outcome = put_local(&st, mailbox_id, id, ts, exp, Some(ct.to_vec())).await.map_err(local)?;
                if outcome == PutOutcome::Stored {
                    presence::after_store(&st, mailbox_id, id, ts, ct.to_vec(), push_id);
                }
                return Ok(());
            }
            let mut body = json!({"mailbox_id": b64u_encode(&mailbox_id), "id": b64u_encode(&id), "ts": ts, "exp": exp, "ct": b64u_encode(&ct)});
            if let Some(push_id) = &push_id {
                body["push_id"] = json!(push_id);
            }
            peer_post(&st, &node, "/n/put", &body).await.map(|_| ())
        }
    });
    let (mut acks, mut full, mut pow) = (0, false, None);
    while let Some(result) = rx.recv().await {
        match result {
            Ok(()) => {
                acks += 1;
                if acks >= plan.quorum {
                    return Ok(acks);
                }
            }
            Err(OpError::Full) => full = true,
            Err(OpError::PowRequired(bits)) => pow = pow.max(Some(bits)),
            Err(OpError::Failed(reason)) => tracing::debug!(%reason, "anchor write failed"),
        }
    }
    Err(match (pow, full) {
        (Some(bits), _) => DataError::PowRequired(bits),
        (None, true) => DataError::MailboxFull,
        (None, false) => DataError::QuorumUnavailable,
    })
}

/// Nedoručené zprávy mailboxu: sjednocení odpovědí nejméně `r − quorum + 1` anchorů.
pub async fn fetch(st: &Arc<NodeState>, mailbox_id: [u8; 32], limit: usize) -> Result<Vec<Stored>, DataError> {
    let plan = plan(st, &mailbox_id)?;
    let need = plan.read_quorum();
    let mut rx = fan_out(st, plan.nodes, move |st, node, is_self| async move {
        if is_self {
            return get_local(&st, mailbox_id, limit).await.map_err(local);
        }
        let body = json!({"mailbox_id": b64u_encode(&mailbox_id), "limit": limit});
        let answer = peer_post(&st, &node, "/n/get", &body).await?;
        let items = answer["messages"].as_array().ok_or_else(|| OpError::Failed("malformed answer".into()))?;
        items.iter().map(|m| Stored::from_json(m).ok_or_else(|| OpError::Failed("malformed message".into()))).collect()
    });
    let (mut answers, mut merged) = (0, BTreeMap::new());
    while let Some(result) = rx.recv().await {
        if let Ok(list) = result {
            answers += 1;
            for m in list {
                merged.entry(m.id).or_insert(m);
            }
            if answers >= need {
                let mut all: Vec<Stored> = merged.into_values().collect();
                all.sort_by_key(|m| (m.ts, m.id));
                all.truncate(limit);
                return Ok(all);
            }
        }
    }
    Err(DataError::QuorumUnavailable)
}

/// Potvrzení doručení na všech anchorech; uspěje při `quorum` odpovědích.
pub async fn ack(st: &Arc<NodeState>, mailbox_id: [u8; 32], ids: Vec<[u8; 16]>) -> Result<(), DataError> {
    let plan = plan(st, &mailbox_id)?;
    let ids = Arc::new(ids);
    let mut rx = fan_out(st, plan.nodes, move |st, node, is_self| {
        let ids = ids.clone();
        async move {
            if is_self {
                return ack_local(&st, mailbox_id, ids.to_vec()).await.map(|_| ()).map_err(local);
            }
            let list: Vec<String> = ids.iter().map(|i| b64u_encode(i)).collect();
            peer_post(&st, &node, "/n/ack", &json!({"mailbox_id": b64u_encode(&mailbox_id), "ids": list})).await.map(|_| ())
        }
    });
    let mut acks = 0;
    while let Some(result) = rx.recv().await {
        if result.is_ok() {
            acks += 1;
            if acks >= plan.quorum {
                return Ok(());
            }
        }
    }
    Err(DataError::QuorumUnavailable)
}

async fn push_record(st: &Arc<NodeState>, node: &DirectoryNode, rec: &Record) -> bool {
    let mut body = json!({
        "mailbox_id": b64u_encode(&rec.mailbox_id),
        "id": b64u_encode(&rec.id),
        "ts": rec.ts,
        "exp": rec.exp,
    });
    match &rec.ct {
        Some(ct) => body["ct"] = json!(b64u_encode(ct)),
        None => body["delivered"] = json!(true),
    }
    peer_post(st, node, "/n/put", &body).await.is_ok()
}

/// Handoff po změně epochy (`spec/PROTOCOL.md` §13.5). Vrací `true`, když se vše podařilo.
pub async fn handoff(st: &Arc<NodeState>) -> bool {
    let Some(dir) = st.directory() else { return true };
    let prev = st.prev_directory();
    let me = st.identity.node_id().to_owned();
    let quorum = usize::try_from(dir.params.quorum).unwrap_or(usize::MAX);
    let Ok(mailboxes) = blocking(st, |s| s.store.mailboxes()).await else { return false };
    let now = unix_now();
    let mut all_ok = true;

    for mailbox_id in mailboxes {
        let current = anchors_in(&dir, &mailbox_id);
        let still_anchor = current.iter().any(|n| n.id == me);
        // kdo se anchorem teprve stal, nemá nic; ostatní už data z minula mají
        let targets: Vec<DirectoryNode> = current
            .iter()
            .filter(|n| n.id != me)
            .filter(|n| !still_anchor || prev.as_ref().is_none_or(|p| !anchors_in(p, &mailbox_id).iter().any(|a| a.id == n.id)))
            .cloned()
            .collect();
        if targets.is_empty() && still_anchor {
            continue;
        }
        let Ok(records) = blocking(st, move |s| s.store.records(&mailbox_id)).await else {
            all_ok = false;
            continue;
        };
        let mut done = Vec::new();
        for rec in records.into_iter().filter(|r| r.exp > now) {
            let mut accepted = 0;
            for node in &targets {
                if push_record(st, node, &rec).await {
                    accepted += 1;
                }
            }
            if still_anchor {
                if accepted < targets.len() {
                    all_ok = false;
                }
            } else if accepted >= quorum.min(current.len()) {
                done.push(rec.id);
            } else {
                all_ok = false;
            }
        }
        if !done.is_empty() && blocking(st, move |s| s.store.remove(&mailbox_id, &done)).await.is_err() {
            all_ok = false;
        }
    }
    all_ok &= crate::profile::handoff(st).await;
    all_ok &= crate::guard::handoff(st).await;
    all_ok
}
