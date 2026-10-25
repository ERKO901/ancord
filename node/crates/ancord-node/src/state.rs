//! Sdílený stav běžícího nodu.

use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::time::Duration;
use std::sync::{Arc, Mutex, RwLock};

use crate::challenge::space::SpaceTable;
use crate::config::Config;
use crate::error::NodeError;
use crate::identity::Identity;
use crate::store::Store;
use crate::sysinfo::LoadMeter;
use crate::ticket::{DirectoryBody, TicketGuard};
use crate::worker::WorkerClient;

pub struct NodeState {
    pub cfg: Config,
    pub identity: Identity,
    pub worker_key: [u8; 32],
    pub directory: RwLock<Option<Arc<DirectoryBody>>>,
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
