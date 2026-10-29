//! Sdílený stav běžícího nodu.

use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::time::Duration;
use std::sync::{Arc, Mutex, RwLock};

use crate::challenge::space::SpaceTable;
use crate::config::Config;
use crate::error::NodeError;
use crate::identity::Identity;
use crate::peer::InSessions;
use crate::presence::{PresenceEntry, Subscribers};
use crate::store::Store;
use crate::sysinfo::LoadMeter;
use crate::ticket::{DirectoryBody, TicketGuard};
use crate::worker::WorkerClient;

/// Poslední předaný signál psaní podle `(mailbox_id, role)`: stav a čas.
pub type TypingState = HashMap<([u8; 32], u8), (bool, u64)>;

pub struct NodeState {
    pub cfg: Config,
    pub identity: Identity,
    pub worker_key: [u8; 32],
    pub directory: RwLock<Option<Arc<DirectoryBody>>>,
    /// Podepsaná obálka aktuálního adresáře, jak ji poslal Worker (předává se peerům, spec §6.1).
    pub directory_envelope: RwLock<Option<Arc<serde_json::Value>>>,
    /// Adresář předchozí epochy: peeři s ním ještě mohou počítat anchory (spec §13.4).
    pub prev_directory: RwLock<Option<Arc<DirectoryBody>>>,
    /// Mailboxy, které tento node drží jako anchor.
    pub store: Store,
    /// Klient pro node↔node requesty.
    pub peer_http: reqwest::Client,
    /// Epocha, pro kterou doběhl handoff (`u64::MAX` = žádná).
    pub handoff_epoch: AtomicU64,
    pub tickets: TicketGuard,
    /// Počet aktivních spojení klientů (hlášeno v heartbeatu).
    pub conns: AtomicU64,
    /// Aktuální commitmenty Proof of Space podle vrstvy (`ram`, `storage`).
    pub spaces: Mutex<HashMap<String, Arc<SpaceTable>>>,
    pub load: LoadMeter,
    /// Nonce přijatých podepsaných requestů od jiných nodů (ochrana proti replayi) s časem vypršení.
    pub seen_nonces: Mutex<HashMap<String, u64>>,
    /// Hostname přidělený Workerem (známe ho po registraci; proof server ho potřebuje pro podpis).
    pub hostname: RwLock<Option<String>>,
    /// Klient Workeru pro `/wake`.
    pub worker: WorkerClient,
    /// Kdy se naposledy poslal wake pro dané `push_id` (lokální koalescence).
    pub wake_last: Mutex<HashMap<String, u64>>,
    /// Presence mailboxů, které tento node drží jako anchor: kde je příjemce online (jen v paměti, spec §13.7).
    pub presence: Mutex<HashMap<[u8; 32], PresenceEntry>>,
    /// Odběratelé z připojených klientů: tento node je jejich gateway.
    pub subscribers: Mutex<HashMap<[u8; 32], Subscribers>>,
    /// Kdy a s jakým stavem se naposledy předal signál psaní daného mailboxu (koalescence, spec §15.5).
    pub typing_last: Mutex<TypingState>,
    /// Čítač spojení klientů (identifikace odběratelů).
    pub next_conn: AtomicU64,
    /// Session node↔node, které tento node drží jako responder (`/n/hs`).
    pub n2n_in: InSessions,
    /// Session node↔node, které tento node otevřel jako iniciátor.
    pub n2n_out: crate::n2n::OutSessions,
    /// Neúspěšná volání peerů pro hlášení v heartbeatu (spec §6.1).
    pub reach: crate::reach::Reach,
}

impl NodeState {
    pub fn new(cfg: Config, identity: Identity, worker_key: [u8; 32]) -> Result<Arc<Self>, NodeError> {
        std::fs::create_dir_all(&cfg.data_dir)?;
        let store = Store::open(&cfg.data_dir.join("mailboxes.redb")).map_err(|e| NodeError(e.to_string()))?;
        let peer_http = reqwest::Client::builder()
            .timeout(Duration::from_secs(cfg.peer_timeout_s))
            .user_agent(concat!("ancord-node/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| NodeError(e.to_string()))?;
        let worker = WorkerClient::new(&cfg.worker_url).map_err(|e| NodeError(e.to_string()))?;
        Ok(Arc::new(Self {
            cfg,
            identity,
            worker_key,
            directory: RwLock::new(None),
            directory_envelope: RwLock::new(None),
            prev_directory: RwLock::new(None),
            store,
            peer_http,
            handoff_epoch: AtomicU64::new(u64::MAX),
            tickets: TicketGuard::default(),
            conns: AtomicU64::new(0),
            spaces: Mutex::new(HashMap::new()),
            load: LoadMeter::default(),
            seen_nonces: Mutex::new(HashMap::new()),
            hostname: RwLock::new(None),
            worker,
            wake_last: Mutex::new(HashMap::new()),
            presence: Mutex::new(HashMap::new()),
            subscribers: Mutex::new(HashMap::new()),
            typing_last: Mutex::default(),
            next_conn: AtomicU64::new(0),
            n2n_in: Mutex::default(),
            n2n_out: Mutex::default(),
            reach: crate::reach::Reach::default(),
        }))
    }

    pub fn hostname(&self) -> Option<String> {
        self.hostname.read().ok().and_then(|h| h.clone())
    }

    pub fn set_hostname(&self, hostname: &str) {
        if let Ok(mut h) = self.hostname.write() {
            *h = Some(hostname.to_owned());
        }
    }

    pub fn directory(&self) -> Option<Arc<DirectoryBody>> {
        self.directory.read().ok().and_then(|d| d.clone())
    }

    /// Uloží nový adresář, jen když je novější než ten, který už držíme.
    pub fn set_directory(&self, dir: DirectoryBody) -> bool {
        let Ok(mut slot) = self.directory.write() else {
            return false;
        };
        if slot.as_ref().is_some_and(|cur| cur.epoch >= dir.epoch) {
            return false;
        }
        let old = slot.replace(Arc::new(dir));
        if let Ok(mut prev) = self.prev_directory.write() {
            *prev = old;
        }
        true
    }

    /// Ověří podpis obálky adresáře klíčem Workeru a převezme ji, je-li novější. Vrací novou epochu.
    pub fn accept_directory(&self, envelope: &serde_json::Value) -> Option<u64> {
        let dir = crate::ticket::verify_directory(&self.worker_key, envelope)?;
        let epoch = dir.epoch;
        if !self.set_directory(dir) {
            return None;
        }
        if let Ok(mut slot) = self.directory_envelope.write() {
            *slot = Some(Arc::new(envelope.clone()));
        }
        Some(epoch)
    }

    pub fn directory_envelope(&self) -> Option<Arc<serde_json::Value>> {
        self.directory_envelope.read().ok().and_then(|d| d.clone())
    }

    pub fn prev_directory(&self) -> Option<Arc<DirectoryBody>> {
        self.prev_directory.read().ok().and_then(|d| d.clone())
    }

    pub fn epoch(&self) -> Option<u64> {
        self.directory().map(|d| d.epoch)
    }

    /// Zapíše nonce požadavku. `false` = už byl použit (replay).
    pub fn consume_nonce(&self, key: String, expires_at: u64, now: u64) -> bool {
        let Ok(mut seen) = self.seen_nonces.lock() else {
            return false;
        };
        seen.retain(|_, exp| *exp > now);
        if seen.contains_key(&key) {
            return false;
        }
        seen.insert(key, expires_at);
        true
    }
}
