//! Sdílený stav běžícího nodu.

use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex, RwLock};

use crate::challenge::space::SpaceTable;
use crate::config::Config;
use crate::identity::Identity;
use crate::sysinfo::LoadMeter;
use crate::ticket::{DirectoryBody, TicketGuard};

pub struct NodeState {
    pub cfg: Config,
    pub identity: Identity,
    pub worker_key: [u8; 32],
    pub directory: RwLock<Option<Arc<DirectoryBody>>>,
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
}

impl NodeState {
    pub fn new(cfg: Config, identity: Identity, worker_key: [u8; 32]) -> Arc<Self> {
        Arc::new(Self {
            cfg,
            identity,
            worker_key,
            directory: RwLock::new(None),
            tickets: TicketGuard::default(),
            conns: AtomicU64::new(0),
            spaces: Mutex::new(HashMap::new()),
            load: LoadMeter::default(),
            seen_nonces: Mutex::new(HashMap::new()),
            hostname: RwLock::new(None),
        })
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
        *slot = Some(Arc::new(dir));
        true
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
