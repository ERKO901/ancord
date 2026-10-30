//! Stav klienta za příkazy. Identita, kontakty, konverzace (stav ratchetu), zprávy a nastavení jsou v šifrovaném
//! úložišti (`store.rs`, D2); `Core::default()` je čistě paměťový a bez sítě (testy UI).
//!
//! Online klient (D3, §13.7): síťová úloha (`net.rs`) je jediný vlastník stavu ratchetu za běhu (`Online`). Se `Core`
//! sdílí `Inner` pod zámkem, který se nikdy nedrží přes `await`. Zprávy z UI jdou do úložiště jako `queued`
//! a úloha je odešle, jakmile je spojení.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ancord_core::encoding::{b64u_decode_array, b64u_encode, hex_encode};
use ancord_core::message::{clean_name, pack};
use ancord_core::net::{CliError, Session};
use ancord_core::online::Persist;
use ancord_core::user::{address_uri, parse_address, safety_number, UserKeys};
use rand::RngCore;
use zeroize::Zeroizing;

use crate::net::{Cmd, Link, NetConfig, Sink};
use crate::store::{AutoDeviceKey, DeviceKey, FileDeviceKey, KdfParams, Store, StoreError, Table};
use serde::{Deserialize, Serialize};

use crate::contract::{
    BiometricStatus,
    ConnectionState, Contact, ConversationPage, ConversationSummary, DeliveryState, IdentityState, IdentityStatus, Message,
    Settings, UiEvent,
};

/// Odeslaná zpráva, jejíž stav se ještě sleduje: `key` je klíč záznamu v `messages`, `sent` je `id` v síti (b64u),
/// dokud není odeslaná, je `None` (`queued`).
#[derive(Serialize, Deserialize, Clone, Debug)]
struct OutItem {
    key: String,
    #[serde(default)]
    sent: Option<String>,
}

struct StoredContact {
    pk: [u8; 32],
    contact: Contact,
    /// Jméno, které kontaktu dal uživatel (prázdné = žádné). Má přednost před jménem od protistrany.
    label: String,
    unread: u32,
    outbox: Vec<OutItem>,
    /// Ještě se má odeslat úvodní rámec s mým jménem (bez textu), který protistrana uvidí v žádosti.
    intro: bool,
}

impl StoredContact {
    fn new(pk: [u8; 32], label: String, peer_name: Option<String>, verified: bool, unread: u32, outbox: Vec<OutItem>) -> Self {
        let mut c = Self {
            pk,
            contact: Contact { id: conv_id(&pk), name: String::new(), address: address_uri(&pk), verified, peer_name, incoming: false },
            label,
            unread,
            outbox,
            intro: false,
        };
        c.refresh_name();
        c
    }

    /// Zobrazované jméno: jméno od uživatele, jinak jméno od protistrany, jinak začátek klíče (než protistrana odpoví).
    fn refresh_name(&mut self) {
        self.contact.name = if !self.label.is_empty() {
            self.label.clone()
        } else if let Some(n) = &self.contact.peer_name {
            n.clone()
        } else {
            short_key(&self.pk)
        };
    }
}

/// Zástupné jméno kontaktu, který se ještě neozval: začátek jeho adresy.
fn short_key(pk: &[u8; 32]) -> String {
    format!("{}…", b64u_encode(pk).chars().take(8).collect::<String>())
}

/// Záznam kontaktu v úložišti; klíč záznamu je `pk`. Kontakt a konverzace s ním mají stejné `id`.
#[derive(Serialize, Deserialize)]
struct ContactRec {
    /// Jméno od uživatele (starší záznamy tu mají zástupné „Kontakt N“, to se při načtení zahodí).
    name: String,
    #[serde(default)]
    peer_name: Option<String>,
    verified: bool,
    #[serde(default)]
    unread: u32,
    #[serde(default)]
    outbox: Vec<OutItem>,
    #[serde(default)]
    incoming: bool,
    #[serde(default)]
    intro: bool,
}

const IDENTITY_KEY: &[u8] = b"keys";
const SETTINGS_KEY: &[u8] = b"app";
const NAME_KEY: &[u8] = b"name";
/// Nejdelší zpráva v bajtech UTF-8.
const MAX_TEXT: usize = 16 * 1024;

#[derive(Default)]
pub(crate) struct Inner {
    keys: Option<UserKeys>,
    locked: bool,
    /// Profil existuje (v úložišti, nebo v paměti). V režimu s heslem je zamčený profil bez `keys`.
    has_profile: bool,
    passphrase: bool,
    contacts: Vec<StoredContact>,
    settings: Settings,
    /// Jméno, které se odesílá ve zprávách (prázdné = žádné).
    display_name: String,
    store: Option<Arc<Store>>,
    conn: ConnectionState,
}

/// Stav sdílený mezi příkazy a síťovou úlohou.
#[derive(Clone, Default)]
pub(crate) struct Shared(Arc<Mutex<Inner>>);

type Network = (Arc<NetConfig>, Arc<dyn Sink>);

/// Odeslané zprávy jedné konverzace, které čekají na doručení: `(klíč záznamu, id v síti)`.
pub(crate) type Awaiting = Vec<(String, [u8; 16])>;

/// Obálka hlavního klíče klíčem, který vydá jen biometrika (na Androidu `BiometricKey.kt`). Volání blokují, dokud uživatel
/// nepotvrdí otiskem nebo obličejem, a nesmí běžet s drženým zámkem stavu ani na hlavním vlákně.
pub trait BiometricKey: Send + Sync {
    fn available(&self) -> bool;
    fn wrap(&self, plain: &[u8]) -> Result<Vec<u8>, String>;
    fn unwrap(&self, blob: &[u8]) -> Result<Zeroizing<Vec<u8>>, String>;
    /// Smaže klíč z Keystore.
    fn forget(&self);
}

const BIO_FILE: &str = "bio.blob";

#[derive(Default)]
pub struct Core {
    shared: Shared,
    bio: Option<Arc<dyn BiometricKey>>,
    /// Složka profilu; `None` = jen paměť.
    dir: Option<PathBuf>,
    kdf: KdfParams,
    /// Použít platformní úložiště klíčů (v testech vypnuto).
    #[cfg_attr(target_os = "android", allow(dead_code, reason = "Android má Keystore přes Kotlin plugin (D6)"))]
    keychain: bool,
    /// Kam se připojit a komu hlásit události; `None` = offline (bez konfigurace sítě nebo v testech).
    network: Mutex<Option<Network>>,
    /// Běžící síťová úloha.
    link: Mutex<Option<Link>>,
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX)).unwrap_or(0)
}

/// `id` kontaktu i konverzace s ním.
pub(crate) fn conv_id(pk: &[u8; 32]) -> String {
    hex_encode(&pk[..8])
}

/// `id` konverzace podle adresy protistrany v `Session` (konverzace klienta jsou vždy podle adresy, §15.4).
pub(crate) fn session_conv(session: &Session) -> Option<String> {
    session.peer.as_deref().and_then(b64u_decode_array::<32>).map(|pk| conv_id(&pk))
}

/// Klíč záznamu zprávy: `conv/ts/id` (§13.6), čas v ms v šestnáctkové soustavě s pevnou délkou, aby se řadil.
fn msg_key(conv: &str, ts: u64, id: &str) -> String {
    format!("{conv}/{ts:016x}/{id}")
}

fn lock_err() -> String {
    "Interní chyba (zámek stavu)".to_string()
}

impl Shared {
    fn lock(&self) -> Result<MutexGuard<'_, Inner>, String> {
        self.0.lock().map_err(|_| lock_err())
    }

    fn store(&self) -> Result<(MutexGuard<'_, Inner>, Arc<Store>), CliError> {
        let g = self.0.lock().map_err(|_| CliError::Session(lock_err()))?;
        let store = g.store.clone().ok_or_else(|| CliError::Session("profile is locked".into()))?;
        Ok((g, store))
    }

    /// Klíče a konverzace pro síťovou úlohu. Klíče jsou kopie: síťová úloha je od teď jediná, kdo je mění a ukládá.
    pub(crate) fn load_online(&self) -> Result<(UserKeys, Vec<Session>), String> {
        let g = self.lock()?;
        let keys = g.keys.as_ref().and_then(|k| UserKeys::from_json(&zeroize::Zeroizing::new(k.to_json()))).ok_or("Identita není odemčená")?;
        let store = g.store.clone().ok_or("Profil není trvalý")?;
        let mut sessions = Vec::new();
        for (_, v) in store.scan(Table::Conversations, b"").map_err(|e| e.to_string())? {
            sessions.push(serde_json::from_slice::<Session>(&v).map_err(|_| StoreError::Corrupt.to_string())?);
        }
        Ok((keys, sessions))
    }

    pub(crate) fn set_conn(&self, state: ConnectionState) -> bool {
        let Ok(mut g) = self.lock() else { return false };
        let changed = g.conn != state;
        g.conn = state;
        changed
    }

    /// Kontakty, se kterými ještě není konverzace (`have` jsou `id` existujících); vrací jejich klíče.
    pub(crate) fn contacts_without(&self, have: &[String]) -> Vec<[u8; 32]> {
        let Ok(g) = self.lock() else { return Vec::new() };
        g.contacts.iter().filter(|c| !have.contains(&c.contact.id)).map(|c| c.pk).collect()
    }

    /// Zprávy čekající na odeslání: `(konverzace, klíč záznamu, text)`, v pořadí zařazení.
    pub(crate) fn queued(&self) -> Vec<(String, String, zeroize::Zeroizing<Vec<u8>>)> {
        let Ok((g, store)) = self.store() else { return Vec::new() };
        let name = (!g.display_name.is_empty()).then_some(g.display_name.as_str());
        let mut out = Vec::new();
        for c in &g.contacts {
            for item in c.outbox.iter().filter(|i| i.sent.is_none()) {
                let Ok(Some(raw)) = store.get(Table::Messages, item.key.as_bytes()) else { continue };
                let Ok(rec) = serde_json::from_slice::<Message>(&raw) else { continue };
                out.push((c.contact.id.clone(), item.key.clone(), zeroize::Zeroizing::new(pack(name, &rec.text))));
            }
        }
        out
    }

    /// Kontakty, kterým se má poslat úvodní rámec se jménem: `(konverzace, plaintext)`. Dokud není jméno nastavené, čeká se.
    pub(crate) fn intros(&self) -> Vec<(String, zeroize::Zeroizing<Vec<u8>>)> {
        let Ok(g) = self.lock() else { return Vec::new() };
        let Some(name) = Some(g.display_name.as_str()).filter(|n| !n.is_empty()) else { return Vec::new() };
        g.contacts.iter().filter(|c| c.intro && !c.contact.incoming).map(|c| (c.contact.id.clone(), zeroize::Zeroizing::new(pack(Some(name), "")))).collect()
    }

    /// Úvodní rámec odešel.
    pub(crate) fn intro_done(&self, conv: &str) -> Result<(), CliError> {
        let (mut g, store) = self.store()?;
        if let Some(c) = g.contacts.iter_mut().find(|c| c.contact.id == conv) {
            c.intro = false;
            save_contact_rec(&store, c).map_err(CliError::Session)?;
        }
        Ok(())
    }

    /// Zpráva odešla s `id` v síti: stav `pending`.
    pub(crate) fn mark_sent(&self, conv: &str, key: &str, net_id: &[u8; 16]) -> Result<Option<UiEvent>, CliError> {
        let (mut g, store) = self.store()?;
        let Some(c) = g.contacts.iter_mut().find(|c| c.contact.id == conv) else { return Ok(None) };
        let Some(item) = c.outbox.iter_mut().find(|i| i.key == key) else { return Ok(None) };
        item.sent = Some(b64u_encode(net_id));
        let event = update_message(&store, conv, key, DeliveryState::Pending)?;
        save_contact_rec(&store, c).map_err(CliError::Session)?;
        Ok(event)
    }

    /// Odeslané zprávy, u kterých se čeká na doručení: konverzace → `(klíč záznamu, id v síti)`.
    pub(crate) fn awaiting(&self) -> Vec<(String, Awaiting)> {
        let Ok(g) = self.lock() else { return Vec::new() };
        g.contacts
            .iter()
            .map(|c| {
                let items = c.outbox.iter().filter_map(|i| Some((i.key.clone(), i.sent.as_deref().and_then(b64u_decode_array::<16>)?))).collect();
                (c.contact.id.clone(), items)
            })
            .filter(|(_, items): &(String, Awaiting)| !items.is_empty())
            .collect()
    }

    pub(crate) fn has_awaiting(&self) -> bool {
        self.lock().is_ok_and(|g| g.contacts.iter().any(|c| c.outbox.iter().any(|i| i.sent.is_some())))
    }

    /// Konečné stavy (`delivered`, `unknown`): zapíše je a přestane zprávy sledovat.
    pub(crate) fn finish(&self, conv: &str, done: &[(String, DeliveryState)]) -> Result<Vec<UiEvent>, CliError> {
        let (mut g, store) = self.store()?;
        let Some(c) = g.contacts.iter_mut().find(|c| c.contact.id == conv) else { return Ok(Vec::new()) };
        let mut events = Vec::new();
        for (key, state) in done {
            events.extend(update_message(&store, conv, key, *state)?);
            c.outbox.retain(|i| &i.key != key);
        }
        save_contact_rec(&store, c).map_err(CliError::Session)?;
        Ok(events)
    }
}

/// Změní stav uložené zprávy; vrací událost pro UI (nebo nic, když zpráva mezitím zmizela).
fn update_message(store: &Store, conv: &str, key: &str, state: DeliveryState) -> Result<Option<UiEvent>, CliError> {
    let err = |e: StoreError| CliError::Session(e.to_string());
    let Some(raw) = store.get(Table::Messages, key.as_bytes()).map_err(err)? else { return Ok(None) };
    let mut msg: Message = serde_json::from_slice(&raw).map_err(|_| err(StoreError::Corrupt))?;
    msg.state = state;
    put_message(store, key, &msg).map_err(CliError::Session)?;
    Ok(Some(UiEvent::DeliveryState { conversation_id: conv.to_owned(), message_id: msg.id, state }))
}

fn put_message(store: &Store, key: &str, msg: &Message) -> Result<(), String> {
    let json = zeroize::Zeroizing::new(serde_json::to_vec(msg).map_err(|e| e.to_string())?);
    store.put(Table::Messages, key.as_bytes(), &json).map_err(|e| e.to_string())
}

fn save_contact_rec(store: &Store, c: &StoredContact) -> Result<(), String> {
    let rec = ContactRec { name: c.label.clone(), peer_name: c.contact.peer_name.clone(), verified: c.contact.verified, unread: c.unread, outbox: c.outbox.clone(), incoming: c.contact.incoming, intro: c.intro };
    let json = serde_json::to_vec(&rec).map_err(|e| e.to_string())?;
    store.put(Table::Contacts, &c.pk, &json).map_err(|e| e.to_string())
}

/// Úložiště pro `Online` (síťová úloha). Přijaté zprávy z historie `Session` zapíše jako záznamy `messages` dřív, než
/// `Online` pošle `ack` (doručení *at least once*, §13.4), a stav ratchetu uloží bez historie. Události pro UI sbírá,
/// síťová úloha je odešle po dokončení operace a historii konverzací pak vyprázdní.
pub(crate) struct StorePersist {
    shared: Shared,
    events: Vec<UiEvent>,
    /// Přijaté zprávy (`id` v síti), které už tato operace zapsala.
    written: HashSet<String>,
}

impl StorePersist {
    pub(crate) fn new(shared: Shared) -> Self {
        Self { shared, events: Vec::new(), written: HashSet::new() }
    }

    /// Události od poslední operace. Volající musí zároveň vyprázdnit `history` konverzací.
    pub(crate) fn take_events(&mut self) -> Vec<UiEvent> {
        self.written.clear();
        std::mem::take(&mut self.events)
    }

    fn save(&mut self, session: &Session, new_contact: bool) -> Result<(), CliError> {
        let conv = session_conv(session).ok_or_else(|| CliError::Session("conversation without a peer address".into()))?;
        let pk = session.peer.as_deref().and_then(b64u_decode_array::<32>).unwrap_or_default();
        let (mut g, store) = self.shared.store()?;
        let err = |e: String| CliError::Session(e);
        // jméno, které protistrana uvedla v nejnovější přijaté zprávě (spec §15.14)
        let peer_name = session.history.iter().rev().filter(|h| !h.outgoing).find_map(|h| h.name.clone());
        if new_contact && !g.contacts.iter().any(|c| c.pk == pk) {
            let mut c = StoredContact::new(pk, String::new(), peer_name.clone(), false, 0, Vec::new());
            c.contact.incoming = true;
            save_contact_rec(&store, &c).map_err(err)?;
            self.events.push(UiEvent::ContactRequest { contact: c.contact.clone() });
            g.contacts.push(c);
        } else if let Some(c) = g.contacts.iter_mut().find(|c| c.pk == pk) {
            if peer_name.is_some() && c.contact.peer_name != peer_name {
                c.contact.peer_name = peer_name;
                c.refresh_name();
                save_contact_rec(&store, c).map_err(err)?;
                self.events.push(UiEvent::ContactUpdate { contact: c.contact.clone() });
            }
        }
        // odmítnutý kontakt: další zprávy se zahodí
        if !g.contacts.iter().any(|c| c.pk == pk) {
            return Ok(());
        }
        let mut fresh = 0;
        for h in session.history.iter().filter(|h| !h.outgoing) {
            if !self.written.insert(h.id.clone()) {
                continue;
            }
            // úvodní rámec se jménem bez textu (žádost, přijetí): jméno už je zpracované, zpráva to není
            if h.text.is_empty() {
                continue;
            }
            let msg = Message { id: h.id.clone(), ts: now_ms(), mine: false, text: h.text.clone(), state: DeliveryState::Delivered };
            put_message(&store, &msg_key(&conv, msg.ts, &msg.id), &msg).map_err(err)?;
            self.events.push(UiEvent::MessageReceived { conversation_id: conv.clone(), message: msg });
            fresh += 1;
        }
        if fresh > 0 {
            if let Some(c) = g.contacts.iter_mut().find(|c| c.pk == pk) {
                c.unread = c.unread.saturating_add(fresh);
                save_contact_rec(&store, c).map_err(err)?;
            }
        }
        // historie je v `messages`, záznam konverzace nese jen stav ratchetu
        let mut bare = session.clone();
        bare.history.clear();
        let json = zeroize::Zeroizing::new(serde_json::to_vec(&bare).map_err(|e| CliError::Session(e.to_string()))?);
        store.put(Table::Conversations, conv.as_bytes(), &json).map_err(|e| CliError::Session(e.to_string()))?;
        drop(g);
        Ok(())
    }
}

impl Persist for StorePersist {
    fn save_keys(&mut self, keys: &UserKeys) -> Result<(), CliError> {
        let (_g, store) = self.shared.store()?;
        store.put(Table::Identity, IDENTITY_KEY, zeroize::Zeroizing::new(keys.to_json()).as_bytes()).map_err(|e| CliError::Session(e.to_string()))
    }

    fn save_conversation(&mut self, _index: usize, session: &Session) -> Result<(), CliError> {
        self.save(session, false)
    }

    fn add_conversation(&mut self, _index: usize, session: &Session) -> Result<(), CliError> {
        self.save(session, true)
    }
}

impl Core {
    /// Core s trvalým profilem ve složce `dir`. Existující profil bez hesla se otevře hned,
    /// profil s heslem zůstane zamčený do `unlock`.
    pub fn persistent(dir: PathBuf, kdf: KdfParams, keychain: bool) -> Result<Self, String> {
        let core = Core { dir: Some(dir), kdf, keychain, ..Core::default() };
        let path = core.profile_path()?;
        if Store::exists(&path) {
            let passphrase = Store::uses_passphrase(&path).map_err(|e| e.to_string())?;
            let mut g = core.shared.lock()?;
            g.has_profile = true;
            g.passphrase = passphrase;
            if passphrase {
                g.locked = true;
            } else {
                core.open_profile(&mut g, None)?;
            }
            drop(g);
        }
        Ok(core)
    }

    #[cfg_attr(not(any(target_os = "android", test)), allow(dead_code, reason = "biometrika jen na Androidu (D6)"))]
    pub fn with_biometric(mut self, bio: Arc<dyn BiometricKey>) -> Self {
        self.bio = Some(bio);
        self
    }

    /// Zapne síť: od teď (a po každém odemčení) běží online klient a hlásí události do `sink`.
    pub fn go_online(&self, config: NetConfig, sink: Arc<dyn Sink>) -> Result<(), String> {
        *self.network.lock().map_err(|_| lock_err())? = Some((Arc::new(config), sink));
        self.ensure_link()
    }

    /// Jednorázové vyzvednutí po wake (Android, §13.4): vrací počet přijatých zpráv. Odmítne běžet vedle online
    /// úlohy (`go_online`), protože stav ratchetu smí měnit jen jeden vlastník. Volá se z vlákna mimo runtime
    /// (receiver, test), ne z async kontextu.
    #[allow(dead_code, reason = "volá ho Kotlin receiver přes JNI (D6); zatím jen test")]
    pub fn sync_once(&self, config: &NetConfig, sink: Arc<dyn Sink>, limit: Duration) -> Result<usize, String> {
        if self.link.lock().map_err(|_| lock_err())?.as_ref().is_some_and(|l| !l.finished()) {
            return Err("Online klient běží, sync_once není potřeba".into());
        }
        let shared = self.shared.clone();
        tauri::async_runtime::block_on(crate::net::sync_once(shared, config, sink, limit)).map_err(|e| e.to_string())
    }

    /// Spustí síťovou úlohu, je-li k tomu všechno (síť, odemčený trvalý profil) a ještě neběží.
    fn ensure_link(&self) -> Result<(), String> {
        let Some((config, sink)) = self.network.lock().map_err(|_| lock_err())?.clone() else { return Ok(()) };
        let ready = self.with(|s| Ok(s.keys.is_some() && s.store.is_some()))?;
        let mut link = self.link.lock().map_err(|_| lock_err())?;
        if ready && link.as_ref().is_none_or(Link::finished) {
            *link = Some(crate::net::start(self.shared.clone(), config, sink));
        }
        Ok(())
    }

    /// Zastaví síťovou úlohu (zahodí i její kopii klíčů); stav ratchetu je vždy uložený před `ack` i před odesláním.
    fn stop_link(&self) -> Result<(), String> {
        let stopped = self.link.lock().map_err(|_| lock_err())?.take().is_some();
        if stopped && self.shared.set_conn(ConnectionState::Offline) {
            if let Some((_, sink)) = self.network.lock().map_err(|_| lock_err())?.as_ref() {
                sink.emit(UiEvent::ConnectionState { state: ConnectionState::Offline });
            }
        }
        Ok(())
    }

    fn notify(&self, cmd: Cmd) {
        if let Ok(link) = self.link.lock() {
            if let Some(l) = link.as_ref() {
                l.send(cmd);
            }
        }
    }

    fn profile_path(&self) -> Result<PathBuf, String> {
        let dir = self.dir.as_ref().ok_or_else(|| "Profil není trvalý".to_string())?;
        std::fs::create_dir_all(dir).map_err(|e| format!("Složka profilu: {e}"))?;
        Ok(dir.join("profile.redb"))
    }

    fn device_key(&self) -> Result<AutoDeviceKey, String> {
        let dir = self.dir.as_ref().ok_or_else(|| "Profil není trvalý".to_string())?;
        #[cfg(not(target_os = "android"))]
        let keychain: Option<Box<dyn DeviceKey + Send + Sync>> = self.keychain.then(|| {
            Box::new(crate::store::KeychainDeviceKey { service: "cz.melonet.ancord".into(), user: format!("device-key:{}", dir.display()) }) as _
        });
        #[cfg(target_os = "android")]
        let keychain: Option<Box<dyn DeviceKey + Send + Sync>> = Some(Box::new(crate::android_keystore::AndroidKeystoreDeviceKey(dir.join("device.key.ks"))));
        Ok(AutoDeviceKey { file: FileDeviceKey(dir.join("device.key")), keychain })
    }

    /// Otevře úložiště a načte z něj identitu, kontakty a nastavení do paměti.
    fn open_profile(&self, s: &mut Inner, passphrase: Option<&str>) -> Result<(), String> {
        let device = self.device_key()?;
        let store = Store::open(&self.profile_path()?, &device as &dyn DeviceKey, passphrase).map_err(|e| e.to_string())?;
        Self::load_profile(s, store)
    }

    /// Načte z otevřeného úložiště identitu, kontakty a nastavení do paměti.
    fn load_profile(s: &mut Inner, store: Store) -> Result<(), String> {
        let keys = store
            .get(Table::Identity, IDENTITY_KEY)
            .map_err(|e| e.to_string())?
            .and_then(|v| std::str::from_utf8(&v).ok().and_then(UserKeys::from_json))
            .ok_or_else(|| StoreError::Corrupt.to_string())?;
        let mut contacts = Vec::new();
        for (k, v) in store.scan(Table::Contacts, b"").map_err(|e| e.to_string())? {
            let pk: [u8; 32] = k.as_slice().try_into().map_err(|_| StoreError::Corrupt.to_string())?;
            let rec: ContactRec = serde_json::from_slice(&v).map_err(|_| StoreError::Corrupt.to_string())?;
            let legacy = rec.name.strip_prefix("Kontakt ").is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()));
            let label = if legacy { String::new() } else { rec.name };
            let mut c = StoredContact::new(pk, label, rec.peer_name, rec.verified, rec.unread, rec.outbox);
            c.contact.incoming = rec.incoming;
            c.intro = rec.intro;
            contacts.push(c);
        }
        s.settings = match store.get(Table::Settings, SETTINGS_KEY).map_err(|e| e.to_string())? {
            Some(v) => serde_json::from_slice(&v).map_err(|_| StoreError::Corrupt.to_string())?,
            None => Settings::default(),
        };
        s.display_name = match store.get(Table::Settings, NAME_KEY).map_err(|e| e.to_string())? {
            Some(v) => String::from_utf8(v.to_vec()).map_err(|_| StoreError::Corrupt.to_string())?,
            None => String::new(),
        };
        s.passphrase = store.passphrase();
        s.keys = Some(keys);
        s.contacts = contacts;
        s.store = Some(Arc::new(store));
        s.locked = false;
        Ok(())
    }

    fn with<T>(&self, f: impl FnOnce(&mut Inner) -> Result<T, String>) -> Result<T, String> {
        let mut g = self.shared.lock()?;
        f(&mut g)
    }

    pub fn identity_status(&self) -> Result<IdentityStatus, String> {
        self.with(|s| {
            let state = match (s.has_profile, s.locked) {
                (false, _) => IdentityState::None,
                (true, true) => IdentityState::Locked,
                (true, false) => IdentityState::Ready,
            };
            Ok(IdentityStatus { state, passphrase: s.passphrase })
        })
    }

    pub fn identity_create(&self, passphrase: Option<String>) -> Result<String, String> {
        let passphrase = passphrase.filter(|p| !p.is_empty());
        let addr = self.with(|s| {
            if s.has_profile {
                return Err("Identita už existuje".into());
            }
            let keys = UserKeys::generate(now());
            let addr = address_uri(&keys.pk());
            if self.dir.is_some() {
                let device = self.device_key()?;
                let store = Store::create(&self.profile_path()?, &device as &dyn DeviceKey, passphrase.as_deref(), self.kdf).map_err(|e| e.to_string())?;
                store.put(Table::Identity, IDENTITY_KEY, zeroize::Zeroizing::new(keys.to_json()).as_bytes()).map_err(|e| e.to_string())?;
                s.store = Some(Arc::new(store));
            } else if passphrase.is_some() {
                return Err("Režim s heslem vyžaduje trvalý profil".into());
            }
            s.passphrase = passphrase.is_some();
            s.keys = Some(keys);
            s.has_profile = true;
            s.locked = false;
            Ok(addr)
        })?;
        self.ensure_link()?;
        Ok(addr)
    }

    /// Změní heslo profilu (u profilu bez hesla ho nastaví). Vyžaduje odemčený trvalý profil a staré heslo, má-li ho.
    /// Data se nepřešifrovávají, mění se jen obal hlavního klíče, takže biometrické odemykání zůstane platné.
    pub fn passphrase_change(&self, old: Option<String>, new: &str) -> Result<(), String> {
        if new.chars().count() < 8 {
            return Err("Heslo musí mít alespoň 8 znaků".into());
        }
        self.with(|s| {
            ready_keys(s)?;
            let store = s.store.clone().ok_or_else(|| "Heslo vyžaduje trvalý profil".to_string())?;
            let device = self.device_key()?;
            store.change_passphrase(&device as &dyn DeviceKey, old.as_deref(), new, self.kdf).map_err(|e| e.to_string())?;
            s.passphrase = true;
            Ok(())
        })
    }

    pub fn address_get(&self) -> Result<String, String> {
        self.with(|s| ready_keys(s).map(|k| address_uri(&k.pk())))
    }

    pub fn address_qr(&self) -> Result<String, String> {
        let addr = self.address_get()?;
        let code = qrcode::QrCode::new(addr.as_bytes()).map_err(|e| format!("QR: {e}"))?;
        Ok(code.render::<qrcode::render::svg::Color>().min_dimensions(224, 224).quiet_zone(true).build())
    }

    /// Uloží kontakt; online klient s ním hned (nebo po připojení) založí konverzaci (§15.4).
    pub fn contact_add(&self, address: &str, name: Option<String>) -> Result<Contact, String> {
        let pk = parse_address(address.trim()).ok_or_else(|| "Neplatná adresa".to_string())?;
        let contact = self.with(|s| {
            let own = ready_keys(s)?.pk();
            if pk == own {
                return Err("To je vaše vlastní adresa".into());
            }
            if s.contacts.iter().any(|c| c.pk == pk) {
                return Err("Kontakt už existuje".into());
            }
            let label = name.and_then(|n| clean_name(&n)).unwrap_or_default();
            let mut c = StoredContact::new(pk, label, None, false, 0, Vec::new());
            // žádost nese jen mé jméno; protistrana ho uvidí hned, já jeho až po přijetí
            c.intro = true;
            if let Some(store) = &s.store {
                save_contact_rec(store, &c)?;
            }
            let contact = c.contact.clone();
            s.contacts.push(c);
            Ok(contact)
        })?;
        self.notify(Cmd::Flush);
        Ok(contact)
    }

    /// Přejmenuje kontakt jen u uživatele; prázdné jméno vrátí jméno od protistrany (nebo začátek klíče).
    pub fn contact_rename(&self, id: &str, name: &str) -> Result<Contact, String> {
        self.with(|s| {
            ready_keys(s)?;
            let c = s.contacts.iter_mut().find(|c| c.contact.id == id).ok_or_else(|| "Neznámý kontakt".to_string())?;
            c.label = clean_name(name).unwrap_or_default();
            c.refresh_name();
            if let Some(store) = &s.store {
                save_contact_rec(store, c)?;
            }
            Ok(c.contact.clone())
        })
    }

    /// Přijme příchozí žádost: protistrana od nás dostane jméno (úvodní rámec).
    pub fn contact_accept(&self, id: &str) -> Result<Contact, String> {
        let contact = self.with(|s| {
            ready_keys(s)?;
            let c = s.contacts.iter_mut().find(|c| c.contact.id == id).ok_or_else(|| "Neznámý kontakt".to_string())?;
            if c.contact.incoming {
                c.contact.incoming = false;
                c.intro = true;
                if let Some(store) = &s.store {
                    save_contact_rec(store, c)?;
                }
            }
            Ok(c.contact.clone())
        })?;
        self.notify(Cmd::Flush);
        Ok(contact)
    }

    /// Odmítne příchozí žádost: kontakt i zprávy zmizí a další zprávy od něj se zahazují. Protistrana se nic nedozví.
    pub fn contact_decline(&self, id: &str) -> Result<(), String> {
        self.with(|s| {
            ready_keys(s)?;
            let i = s.contacts.iter().position(|c| c.contact.id == id).ok_or_else(|| "Neznámý kontakt".to_string())?;
            if !s.contacts[i].contact.incoming {
                return Err("Kontakt není žádost".into());
            }
            if let Some(store) = &s.store {
                store.delete_prefix(Table::Messages, format!("{id}/").as_bytes()).map_err(|e| e.to_string())?;
                store.delete(Table::Contacts, &s.contacts[i].pk).map_err(|e| e.to_string())?;
            }
            s.contacts.remove(i);
            Ok(())
        })
    }

    /// Jméno, které se posílá ve zprávách (prázdné = nepošle se žádné).
    pub fn display_name_get(&self) -> Result<String, String> {
        self.with(|s| {
            ready_keys(s)?;
            Ok(s.display_name.clone())
        })
    }

    /// Nastaví jméno do dalších zpráv. Dřívější zprávy ho už mají; protistrana ho převezme z první nové.
    pub fn display_name_set(&self, name: &str) -> Result<String, String> {
        self.with(|s| {
            ready_keys(s)?;
            let name = clean_name(name).unwrap_or_default();
            if let Some(store) = &s.store {
                store.put(Table::Settings, NAME_KEY, name.as_bytes()).map_err(|e| e.to_string())?;
            }
            s.display_name = name.clone();
            Ok(name)
        })
    }

    pub fn contact_list(&self) -> Result<Vec<Contact>, String> {
        self.with(|s| {
            ready_keys(s)?;
            Ok(s.contacts.iter().map(|c| c.contact.clone()).collect())
        })
    }

    pub fn contact_safety_number(&self, id: &str) -> Result<String, String> {
        self.with(|s| {
            let own = ready_keys(s)?.pk();
            let c = s.contacts.iter().find(|c| c.contact.id == id).ok_or_else(|| "Neznámý kontakt".to_string())?;
            Ok(safety_number(&own, &c.pk))
        })
    }

    pub fn contact_mark_verified(&self, id: &str) -> Result<(), String> {
        self.with(|s| {
            ready_keys(s)?;
            let c = s.contacts.iter_mut().find(|c| c.contact.id == id).ok_or_else(|| "Neznámý kontakt".to_string())?;
            c.contact.verified = true;
            match &s.store {
                Some(store) => save_contact_rec(store, c),
                None => Ok(()),
            }
        })
    }

    /// Konverzace se zprávami, nejnovější první (kontakt bez zpráv se ukazuje jen v kontaktech).
    pub fn conversation_list(&self) -> Result<Vec<ConversationSummary>, String> {
        self.with(|s| {
            ready_keys(s)?;
            let Some(store) = &s.store else { return Ok(Vec::new()) };
            let mut list = Vec::new();
            for c in &s.contacts {
                let (last, _) = store.scan_rev(Table::Messages, format!("{}/", c.contact.id).as_bytes(), None, 1).map_err(|e| e.to_string())?;
                let Some((_, raw)) = last.first() else { continue };
                let m: Message = serde_json::from_slice(raw).map_err(|_| StoreError::Corrupt.to_string())?;
                list.push(ConversationSummary {
                    id: c.contact.id.clone(),
                    contact_id: c.contact.id.clone(),
                    name: c.contact.name.clone(),
                    last_text: m.text,
                    last_ts: m.ts,
                    unread: c.unread,
                    last_state: m.mine.then_some(m.state),
                });
            }
            list.sort_by_key(|c| std::cmp::Reverse(c.last_ts));
            Ok(list)
        })
    }

    /// Stránka zpráv konverzace, nejvýše `limit` zpráv starších než `before` (ms), ve vzestupném pořadí.
    /// Otevření konverzace vynuluje počet nepřečtených.
    pub fn conversation_open(&self, id: &str, before: Option<u64>, limit: u32) -> Result<ConversationPage, String> {
        self.with(|s| {
            ready_keys(s)?;
            let Some(store) = s.store.clone() else { return Ok(ConversationPage { messages: Vec::new(), more: false }) };
            let c = s.contacts.iter_mut().find(|c| c.contact.id == id).ok_or_else(|| "Neznámá konverzace".to_string())?;
            let prefix = format!("{id}/");
            let before = before.map(|b| format!("{b:016x}"));
            let limit = usize::try_from(limit.clamp(1, 1000)).unwrap_or(1000);
            let (rows, more) = store.scan_rev(Table::Messages, prefix.as_bytes(), before.as_deref().map(str::as_bytes), limit).map_err(|e| e.to_string())?;
            let mut messages = Vec::with_capacity(rows.len());
            for (_, raw) in rows.iter().rev() {
                messages.push(serde_json::from_slice::<Message>(raw).map_err(|_| StoreError::Corrupt.to_string())?);
            }
            if c.unread > 0 {
                c.unread = 0;
                save_contact_rec(&store, c)?;
            }
            Ok(ConversationPage { messages, more })
        })
    }

    /// Uloží zprávu jako `queued` a předá ji síťové úloze; stav se pak mění událostí `delivery_state`.
    pub fn message_send(&self, id: &str, text: &str) -> Result<Message, String> {
        let text = text.trim();
        if text.is_empty() {
            return Err("Prázdná zpráva".into());
        }
        if text.len() > MAX_TEXT {
            return Err("Zpráva je příliš dlouhá".into());
        }
        let msg = self.with(|s| {
            ready_keys(s)?;
            let store = s.store.clone().ok_or_else(|| "Zprávy vyžadují trvalý profil".to_string())?;
            let c = s.contacts.iter_mut().find(|c| c.contact.id == id).ok_or_else(|| "Neznámá konverzace".to_string())?;
            // odpověď na žádost ji přijímá; jméno jde s touto zprávou
            c.contact.incoming = false;
            c.intro = false;
            let mut raw = [0u8; 16];
            rand::rngs::OsRng.fill_bytes(&mut raw);
            let msg = Message { id: b64u_encode(&raw), ts: now_ms(), mine: true, text: text.to_owned(), state: DeliveryState::Queued };
            let key = msg_key(id, msg.ts, &msg.id);
            put_message(&store, &key, &msg)?;
            c.outbox.push(OutItem { key, sent: None });
            save_contact_rec(&store, c)?;
            Ok(msg)
        })?;
        self.notify(Cmd::Flush);
        Ok(msg)
    }

    pub fn typing_set(&self, id: &str, on: bool) {
        self.notify(Cmd::Typing { id: id.to_owned(), on });
    }

    /// Smaže zprávy konverzace (historii) a přestane sledovat jejich doručení. Stav ratchetu zůstává: protistrana
    /// může psát dál a znovu založit konverzaci se stejným kontaktem by nešlo (§9, bod 17).
    pub fn conversation_delete(&self, id: &str) -> Result<(), String> {
        self.with(|s| {
            ready_keys(s)?;
            if let Some(store) = s.store.clone() {
                store.delete_prefix(Table::Messages, format!("{id}/").as_bytes()).map_err(|e| e.to_string())?;
                if let Some(c) = s.contacts.iter_mut().find(|c| c.contact.id == id) {
                    c.outbox.clear();
                    c.unread = 0;
                    save_contact_rec(&store, c)?;
                }
            }
            Ok(())
        })
    }

    pub fn connection_state(&self) -> ConnectionState {
        self.with(|s| Ok(s.conn)).unwrap_or(ConnectionState::Offline)
    }

    pub fn settings_get(&self) -> Result<Settings, String> {
        self.with(|s| Ok(s.settings.clone()))
    }

    pub fn settings_set(&self, settings: Settings) -> Result<(), String> {
        self.with(|s| {
            if let Some(store) = &s.store {
                let json = serde_json::to_vec(&settings).map_err(|e| e.to_string())?;
                store.put(Table::Settings, SETTINGS_KEY, &json).map_err(|e| e.to_string())?;
            }
            s.settings = settings;
            Ok(())
        })
    }

    /// Zamkne UI. V režimu s heslem zmizí z paměti i klíče a úložiště, a proto se zastaví i síť; jinak zprávy chodí dál.
    pub fn lock(&self) -> Result<(), String> {
        let wipe = self.with(|s| Ok(s.has_profile && s.passphrase))?;
        if wipe {
            self.stop_link()?;
        }
        self.with(|s| {
            if s.has_profile {
                s.locked = true;
                if s.passphrase {
                    s.keys = None;
                    s.contacts.clear();
                    s.store = None;
                }
            }
            Ok(())
        })
    }

    fn bio_path(&self) -> Result<PathBuf, String> {
        self.dir.as_ref().map(|d| d.join(BIO_FILE)).ok_or_else(|| "Profil není uložený".to_string())
    }

    pub fn biometric_status(&self) -> Result<BiometricStatus, String> {
        let passphrase = self.with(|s| Ok(s.has_profile && s.passphrase))?;
        let available = passphrase && self.dir.is_some() && self.bio.as_ref().is_some_and(|b| b.available());
        let enabled = available && self.bio_path()?.is_file();
        Ok(BiometricStatus { available, enabled })
    }

    /// Zapne biometrické odemykání; vyžaduje odemčený profil s heslem a jedno potvrzení biometrikou.
    pub fn biometric_enable(&self) -> Result<(), String> {
        let bio = self.bio.clone().ok_or("Biometrika na tomto zařízení není k dispozici")?;
        let store = self.with(|s| match (&s.store, s.passphrase) {
            (Some(st), true) => Ok(st.clone()),
            (_, false) => Err("Biometrika se dá zapnout jen u profilu s heslem".to_string()),
            _ => Err("Aplikace je zamčená".to_string()),
        })?;
        // bez drženého zámku stavu: čeká se na uživatele
        let blob = bio.wrap(store.master_key().as_ref())?;
        let path = self.bio_path()?;
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, &blob).and_then(|()| std::fs::rename(&tmp, &path)).map_err(|e| e.to_string())
    }

    pub fn biometric_disable(&self) -> Result<(), String> {
        if let Some(bio) = &self.bio {
            bio.forget();
        }
        match std::fs::remove_file(self.bio_path()?) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }

    /// Odemkne profil s heslem biometrikou. Při zrušení nebo neplatném klíči (nový otisk) zůstane zamčeno a jde heslem.
    pub fn unlock_biometric(&self) -> Result<(), String> {
        let bio = self.bio.clone().ok_or("Biometrika na tomto zařízení není k dispozici")?;
        let blob = std::fs::read(self.bio_path()?).map_err(|_| "Biometrické odemykání není zapnuté".to_string())?;
        let master = bio.unwrap(&blob)?;
        self.with(|s| {
            if !s.has_profile || !s.passphrase {
                return Err("Biometrika se použije jen u profilu s heslem".to_string());
            }
            if s.store.is_none() {
                let store = Store::open_with_master(&self.profile_path()?, master.as_ref()).map_err(|e| e.to_string())?;
                Self::load_profile(s, store)?;
            }
            s.locked = false;
            Ok(())
        })?;
        self.ensure_link()
    }

    pub fn unlock(&self, passphrase: Option<String>) -> Result<(), String> {
        self.with(|s| {
            if !s.has_profile {
                return Ok(());
            }
            if s.passphrase {
                if s.store.is_none() {
                    let pass = passphrase.as_deref().filter(|p| !p.is_empty()).ok_or_else(|| StoreError::Passphrase.to_string())?;
                    self.open_profile(s, Some(pass))?;
                }
            } else if self.dir.is_some() && s.store.is_none() {
                self.open_profile(s, None)?;
            }
            s.locked = false;
            Ok(())
        })?;
        self.ensure_link()
    }
}

/// Klíče jen u odemčené identity; zamčené UI nedostane ani seznam kontaktů.
fn ready_keys(s: &Inner) -> Result<&UserKeys, String> {
    match (&s.keys, s.has_profile, s.locked) {
        (Some(k), _, false) => Ok(k),
        (_, true, _) => Err("Aplikace je zamčená".into()),
        (_, false, _) => Err("Identita ještě není vytvořena".into()),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn ready() -> Core {
        let c = Core::default();
        c.identity_create(None).unwrap();
        c
    }

    #[test]
    fn identity_is_created_once_and_address_has_the_scheme() {
        let c = Core::default();
        assert_eq!(c.identity_status().unwrap().state, IdentityState::None);
        let addr = c.identity_create(None).unwrap();
        assert!(addr.starts_with("ancord:"));
        assert_eq!(c.address_get().unwrap(), addr);
        assert!(c.identity_create(None).is_err());
        assert_eq!(c.identity_status().unwrap().state, IdentityState::Ready);
    }

    #[test]
    fn contact_shows_the_key_until_the_peer_name_arrives_and_a_label_wins() {
        let dir = tempfile::tempdir().unwrap();
        let a = Core::persistent(dir.path().to_path_buf(), FAST, false).unwrap();
        a.identity_create(None).unwrap();
        let bob = ready().address_get().unwrap();
        let c = a.contact_add(&bob, None).unwrap();
        assert!(c.name.ends_with('…') && bob.contains(c.name.trim_end_matches('…')), "{}", c.name);
        // jméno od protistrany (tak ho zapíše StorePersist při příjmu zprávy)
        a.with(|s| {
            let k = s.contacts.iter_mut().find(|k| k.contact.id == c.id).unwrap();
            k.contact.peer_name = Some("Bob B.".into());
            k.refresh_name();
            save_contact_rec(s.store.as_ref().unwrap(), k)
        })
        .unwrap();
        assert_eq!(a.contact_list().unwrap()[0].name, "Bob B.");
        let renamed = a.contact_rename(&c.id, "  Bobík ").unwrap();
        assert_eq!((renamed.name.as_str(), renamed.peer_name.as_deref()), ("Bobík", Some("Bob B.")));
        drop(a);
        let b = Core::persistent(dir.path().to_path_buf(), FAST, false).unwrap();
        assert_eq!(b.contact_list().unwrap()[0].name, "Bobík");
        assert_eq!(b.contact_rename(&c.id, "").unwrap().name, "Bob B.");
    }

    #[test]
    fn legacy_placeholder_names_are_not_kept_as_labels() {
        let dir = tempfile::tempdir().unwrap();
        let a = Core::persistent(dir.path().to_path_buf(), FAST, false).unwrap();
        a.identity_create(None).unwrap();
        let bob = ready().address_get().unwrap();
        let pk = parse_address(&bob).unwrap();
        a.with(|s| save_contact_rec(s.store.as_ref().unwrap(), &StoredContact::new(pk, "Kontakt 2".into(), None, false, 0, Vec::new()))).unwrap();
        drop(a);
        let b = Core::persistent(dir.path().to_path_buf(), FAST, false).unwrap();
        assert!(b.contact_list().unwrap()[0].name.ends_with('…'));
    }

    #[test]
    fn display_name_is_cleaned_persisted_and_sent_with_queued_messages() {
        let dir = tempfile::tempdir().unwrap();
        let a = Core::persistent(dir.path().to_path_buf(), FAST, false).unwrap();
        a.identity_create(None).unwrap();
        assert_eq!(a.display_name_get().unwrap(), "");
        assert_eq!(a.display_name_set("  Alice\u{202E} ").unwrap(), "Alice");
        let c = a.contact_add(&ready().address_get().unwrap(), None).unwrap();
        a.message_send(&c.id, "ahoj").unwrap();
        let queued = a.shared.queued();
        let content = ancord_core::message::unpack(&queued[0].2);
        assert_eq!((content.name.as_deref(), content.text.as_str()), (Some("Alice"), "ahoj"));
        drop(a);
        let b = Core::persistent(dir.path().to_path_buf(), FAST, false).unwrap();
        assert_eq!(b.display_name_get().unwrap(), "Alice");
        assert_eq!(b.display_name_set("").unwrap(), "");
    }

    #[test]
    fn add_sends_a_name_only_intro_and_an_incoming_request_is_accepted_or_declined() {
        let dir = tempfile::tempdir().unwrap();
        let a = Core::persistent(dir.path().to_path_buf(), FAST, false).unwrap();
        a.identity_create(None).unwrap();
        let c = a.contact_add(&ready().address_get().unwrap(), None).unwrap();
        // bez nastaveného jména se nic neposílá; po nastavení odejde rámec se jménem a bez textu
        assert!(a.shared.intros().is_empty());
        a.display_name_set("Alice").unwrap();
        let intros = a.shared.intros();
        let content = ancord_core::message::unpack(&intros[0].1);
        assert_eq!((intros[0].0.as_str(), content.name.as_deref(), content.text.as_str()), (c.id.as_str(), Some("Alice"), ""));
        a.shared.intro_done(&c.id).unwrap();
        assert!(a.shared.intros().is_empty());

        // příchozí žádost: nic se neposílá, dokud ji uživatel nepřijme; přijetí pošle jméno
        a.with(|s| {
            s.contacts[0].contact.incoming = true;
            s.contacts[0].intro = false;
            Ok(())
        })
        .unwrap();
        assert!(a.contact_list().unwrap()[0].incoming);
        let accepted = a.contact_accept(&c.id).unwrap();
        assert!(!accepted.incoming);
        assert_eq!(a.shared.intros().len(), 1);
        // odmítnout lze jen žádost
        assert!(a.contact_decline(&c.id).is_err());
        a.with(|s| {
            s.contacts[0].contact.incoming = true;
            Ok(())
        })
        .unwrap();
        a.contact_decline(&c.id).unwrap();
        assert!(a.contact_list().unwrap().is_empty());
        drop(a);
        let b = Core::persistent(dir.path().to_path_buf(), FAST, false).unwrap();
        assert!(b.contact_list().unwrap().is_empty());
    }

    #[test]
    fn passphrase_can_be_set_and_changed_and_keeps_the_data() {
        let dir = tempfile::tempdir().unwrap();
        let a = Core::persistent(dir.path().to_path_buf(), FAST, false).unwrap();
        let addr = a.identity_create(None).unwrap();
        assert!(a.passphrase_change(None, "krátké").is_err(), "heslo má mít alespoň 8 znaků");
        // profil bez hesla: heslo se nově nastaví
        a.passphrase_change(None, "prvni-heslo").unwrap();
        assert!(a.identity_status().unwrap().passphrase);
        // změna vyžaduje správné staré heslo
        assert_eq!(a.passphrase_change(None, "druhe-heslo").unwrap_err(), StoreError::Passphrase.to_string());
        assert_eq!(a.passphrase_change(Some("spatne".into()), "druhe-heslo").unwrap_err(), StoreError::Unlock.to_string());
        a.passphrase_change(Some("prvni-heslo".into()), "druhe-heslo").unwrap();
        a.lock().unwrap();
        drop(a);
        let b = Core::persistent(dir.path().to_path_buf(), FAST, false).unwrap();
        assert_eq!(b.identity_status().unwrap().state, IdentityState::Locked);
        assert_eq!(b.unlock(Some("prvni-heslo".into())).unwrap_err(), StoreError::Unlock.to_string());
        b.unlock(Some("druhe-heslo".into())).unwrap();
        assert_eq!(b.address_get().unwrap(), addr);
    }

    #[test]
    fn qr_is_an_svg() {
        let svg = ready().address_qr().unwrap();
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn contacts_reject_garbage_self_and_duplicates() {
        let a = ready();
        let b = ready();
        assert!(a.contact_add("není adresa", None).is_err());
        assert!(a.contact_add(&a.address_get().unwrap(), None).is_err());
        let addr_b = b.address_get().unwrap();
        let c = a.contact_add(&addr_b, Some("  Bob ".into())).unwrap();
        assert_eq!(c.name, "Bob");
        assert!(a.contact_add(&addr_b, None).is_err());
        assert_eq!(a.contact_list().unwrap().len(), 1);
    }

    #[test]
    fn safety_number_is_the_same_on_both_sides() {
        let a = ready();
        let b = ready();
        let ca = a.contact_add(&b.address_get().unwrap(), None).unwrap();
        let cb = b.contact_add(&a.address_get().unwrap(), None).unwrap();
        assert_eq!(a.contact_safety_number(&ca.id).unwrap(), b.contact_safety_number(&cb.id).unwrap());
    }

    #[test]
    fn locked_core_exposes_nothing() {
        let c = ready();
        c.lock().unwrap();
        assert_eq!(c.identity_status().unwrap().state, IdentityState::Locked);
        assert!(c.contact_list().is_err());
        assert!(c.address_get().is_err());
        c.unlock(None).unwrap();
        assert!(c.contact_list().is_ok());
    }

    #[test]
    fn memory_core_cannot_send() {
        let a = ready();
        let c = a.contact_add(&ready().address_get().unwrap(), None).unwrap();
        assert!(a.message_send(&c.id, "ahoj").is_err());
    }

    #[test]
    fn passphrase_mode_needs_a_persistent_profile() {
        assert!(Core::default().identity_create(Some("tajne".into())).is_err());
    }

    const FAST: KdfParams = KdfParams { m_kib: 8, t: 1 };

    #[test]
    fn identity_contacts_and_settings_survive_a_restart() {
        let dir = tempfile::tempdir().unwrap();
        let a = Core::persistent(dir.path().to_path_buf(), FAST, false).unwrap();
        assert_eq!(a.identity_status().unwrap().state, IdentityState::None);
        let addr = a.identity_create(None).unwrap();
        let bob = ready().address_get().unwrap();
        let c = a.contact_add(&bob, Some("Bob".into())).unwrap();
        a.contact_mark_verified(&c.id).unwrap();
        let mut st = a.settings_get().unwrap();
        st.lock_after_secs = 42;
        a.settings_set(st).unwrap();
        drop(a);

        let b = Core::persistent(dir.path().to_path_buf(), FAST, false).unwrap();
        assert_eq!(b.identity_status().unwrap().state, IdentityState::Ready);
        assert_eq!(b.address_get().unwrap(), addr);
        let list = b.contact_list().unwrap();
        assert_eq!((list[0].name.as_str(), list[0].verified), ("Bob", true));
        assert_eq!(b.settings_get().unwrap().lock_after_secs, 42);
    }

    #[test]
    fn offline_messages_are_queued_paged_and_survive_a_restart() {
        let dir = tempfile::tempdir().unwrap();
        let a = Core::persistent(dir.path().to_path_buf(), FAST, false).unwrap();
        a.identity_create(None).unwrap();
        let c = a.contact_add(&ready().address_get().unwrap(), Some("Bob".into())).unwrap();
        assert!(a.conversation_list().unwrap().is_empty());
        assert!(a.message_send(&c.id, "   ").is_err());
        let mut sent = Vec::new();
        for i in 0..5 {
            sent.push(a.message_send(&c.id, &format!("zpráva {i}")).unwrap());
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(sent.iter().all(|m| m.state == DeliveryState::Queued && m.mine));
        assert_eq!(a.shared.queued().len(), 5);
        drop(a);

        let a = Core::persistent(dir.path().to_path_buf(), FAST, false).unwrap();
        let list = a.conversation_list().unwrap();
        assert_eq!((list[0].last_text.as_str(), list[0].last_state), ("zpráva 4", Some(DeliveryState::Queued)));
        let page = a.conversation_open(&c.id, None, 3).unwrap();
        assert_eq!(page.messages.iter().map(|m| m.text.as_str()).collect::<Vec<_>>(), ["zpráva 2", "zpráva 3", "zpráva 4"]);
        assert!(page.more);
        let older = a.conversation_open(&c.id, Some(page.messages[0].ts), 3).unwrap();
        assert_eq!(older.messages.iter().map(|m| m.text.as_str()).collect::<Vec<_>>(), ["zpráva 0", "zpráva 1"]);
        assert!(!older.more);
        assert_eq!(a.shared.queued().len(), 5);

        a.conversation_delete(&c.id).unwrap();
        assert!(a.conversation_list().unwrap().is_empty());
        assert!(a.shared.queued().is_empty());
    }

    #[test]
    fn passphrase_profile_stays_locked_until_the_right_passphrase() {
        let dir = tempfile::tempdir().unwrap();
        let a = Core::persistent(dir.path().to_path_buf(), FAST, false).unwrap();
        let addr = a.identity_create(Some("heslo".into())).unwrap();
        a.lock().unwrap();
        assert!(a.address_get().is_err());
        assert_eq!(a.unlock(Some("spatne".into())).unwrap_err(), StoreError::Unlock.to_string());
        assert_eq!(a.unlock(None).unwrap_err(), StoreError::Passphrase.to_string());
        assert_eq!(a.identity_status().unwrap().state, IdentityState::Locked);
        a.unlock(Some("heslo".into())).unwrap();
        assert_eq!(a.address_get().unwrap(), addr);
        drop(a);

        let b = Core::persistent(dir.path().to_path_buf(), FAST, false).unwrap();
        let st = b.identity_status().unwrap();
        assert_eq!((st.state, st.passphrase), (IdentityState::Locked, true));
        assert!(b.contact_list().is_err());
        b.unlock(Some("heslo".into())).unwrap();
        assert_eq!(b.address_get().unwrap(), addr);
    }

    /// Náhrada Keystore: „biometrika“ jen zamění bajty a dá se vypnout (zrušení dotazu).
    struct FakeBio {
        allow: std::sync::atomic::AtomicBool,
    }

    impl BiometricKey for FakeBio {
        fn available(&self) -> bool {
            true
        }
        fn wrap(&self, plain: &[u8]) -> Result<Vec<u8>, String> {
            self.allow.load(std::sync::atomic::Ordering::SeqCst).then(|| plain.iter().map(|b| b ^ 0x5a).collect()).ok_or_else(|| "zrušeno".to_string())
        }
        fn unwrap(&self, blob: &[u8]) -> Result<Zeroizing<Vec<u8>>, String> {
            self.allow.load(std::sync::atomic::Ordering::SeqCst).then(|| Zeroizing::new(blob.iter().map(|b| b ^ 0x5a).collect())).ok_or_else(|| "zrušeno".to_string())
        }
        fn forget(&self) {}
    }

    #[test]
    fn biometric_unlocks_only_a_passphrase_profile_and_password_still_works() {
        let dir = tempfile::tempdir().unwrap();
        let bio = Arc::new(FakeBio { allow: true.into() });
        let a = Core::persistent(dir.path().to_path_buf(), FAST, false).unwrap().with_biometric(bio.clone());
        // bez profilu s heslem není co zapnout
        assert!(!a.biometric_status().unwrap().available);
        let addr = a.identity_create(Some("heslo".into())).unwrap();
        assert_eq!(a.biometric_status().unwrap(), BiometricStatus { available: true, enabled: false });
        a.biometric_enable().unwrap();
        assert_eq!(a.biometric_status().unwrap(), BiometricStatus { available: true, enabled: true });

        a.lock().unwrap();
        assert!(a.address_get().is_err());
        // zrušená biometrika nic neodemkne, profil zůstane zamčený a heslo funguje
        bio.allow.store(false, std::sync::atomic::Ordering::SeqCst);
        assert!(a.unlock_biometric().is_err());
        assert_eq!(a.identity_status().unwrap().state, IdentityState::Locked);
        bio.allow.store(true, std::sync::atomic::Ordering::SeqCst);
        a.unlock_biometric().unwrap();
        assert_eq!(a.address_get().unwrap(), addr);

        // po restartu aplikace (nové jádro nad stejnou složkou)
        drop(a);
        let b = Core::persistent(dir.path().to_path_buf(), FAST, false).unwrap().with_biometric(bio);
        assert_eq!(b.identity_status().unwrap().state, IdentityState::Locked);
        b.unlock_biometric().unwrap();
        assert_eq!(b.address_get().unwrap(), addr);

        b.biometric_disable().unwrap();
        b.lock().unwrap();
        assert!(b.unlock_biometric().is_err());
        b.unlock(Some("heslo".into())).unwrap();
    }

    #[test]
    fn biometric_is_refused_for_a_profile_without_passphrase() {
        let dir = tempfile::tempdir().unwrap();
        let a = Core::persistent(dir.path().to_path_buf(), FAST, false)
            .unwrap()
            .with_biometric(Arc::new(FakeBio { allow: true.into() }));
        a.identity_create(None).unwrap();
        assert!(!a.biometric_status().unwrap().available);
        assert!(a.biometric_enable().is_err());
    }
}
