//! Stav klienta za příkazy. Milník D2: identita, kontakty a nastavení jsou v šifrovaném úložišti (`store.rs`),
//! `Core::default()` je čistě paměťový (testy, mock). Online session (D3) přijde; do té doby zprávy vracejí chybu.

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use ancord_core::encoding::hex_encode;
use ancord_core::user::{address_uri, parse_address, safety_number, UserKeys};

use crate::store::{AutoDeviceKey, DeviceKey, FileDeviceKey, KdfParams, Store, StoreError, Table};
use serde::{Deserialize, Serialize};

use crate::contract::{
    ConnectionState, Contact, ConversationPage, ConversationSummary, IdentityState, IdentityStatus, Message, Settings,
};

struct StoredContact {
    pk: [u8; 32],
    contact: Contact,
}

/// Záznam kontaktu v úložišti; klíč záznamu je `pk`.
#[derive(Serialize, Deserialize)]
struct ContactRec {
    name: String,
    verified: bool,
}

const IDENTITY_KEY: &[u8] = b"keys";
const SETTINGS_KEY: &[u8] = b"app";

#[derive(Default)]
struct Inner {
    keys: Option<UserKeys>,
    locked: bool,
    /// Profil existuje (v úložišti, nebo v paměti). V režimu s heslem je zamčený profil bez `keys`.
    has_profile: bool,
    passphrase: bool,
    contacts: Vec<StoredContact>,
    settings: Settings,
    store: Option<Store>,
}

#[derive(Default)]
pub struct Core {
    inner: Mutex<Inner>,
    /// Složka profilu; `None` = jen paměť.
    dir: Option<PathBuf>,
    kdf: KdfParams,
    /// Použít platformní úložiště klíčů (v testech vypnuto).
    keychain: bool,
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

const NOT_YET: &str = "Zprávy přijdou v milníku D3 (online klient)";

impl Core {
    /// Core s trvalým profilem ve složce `dir`. Existující profil bez hesla se otevře hned,
    /// profil s heslem zůstane zamčený do `unlock`.
    pub fn persistent(dir: PathBuf, kdf: KdfParams, keychain: bool) -> Result<Self, String> {
        let core = Core { inner: Mutex::default(), dir: Some(dir), kdf, keychain };
        let path = core.profile_path()?;
        if Store::exists(&path) {
            let passphrase = Store::uses_passphrase(&path).map_err(|e| e.to_string())?;
            let mut g = core.inner.lock().map_err(|_| "Interní chyba (zámek stavu)".to_string())?;
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
        let keychain: Option<Box<dyn DeviceKey + Send + Sync>> = None;
        Ok(AutoDeviceKey { file: FileDeviceKey(dir.join("device.key")), keychain })
    }

    /// Otevře úložiště a načte z něj identitu, kontakty a nastavení do paměti.
    fn open_profile(&self, s: &mut Inner, passphrase: Option<&str>) -> Result<(), String> {
        let device = self.device_key()?;
        let store = Store::open(&self.profile_path()?, &device as &dyn DeviceKey, passphrase).map_err(|e| e.to_string())?;
        let keys = store
            .get(Table::Identity, IDENTITY_KEY)
            .map_err(|e| e.to_string())?
            .and_then(|v| std::str::from_utf8(&v).ok().and_then(UserKeys::from_json))
            .ok_or_else(|| StoreError::Corrupt.to_string())?;
        let mut contacts = Vec::new();
        for (k, v) in store.scan(Table::Contacts, b"").map_err(|e| e.to_string())? {
            let pk: [u8; 32] = k.as_slice().try_into().map_err(|_| StoreError::Corrupt.to_string())?;
            let rec: ContactRec = serde_json::from_slice(&v).map_err(|_| StoreError::Corrupt.to_string())?;
            contacts.push(StoredContact { pk, contact: Contact { id: hex_encode(&pk[..8]), name: rec.name, address: address_uri(&pk), verified: rec.verified } });
        }
        s.settings = match store.get(Table::Settings, SETTINGS_KEY).map_err(|e| e.to_string())? {
            Some(v) => serde_json::from_slice(&v).map_err(|_| StoreError::Corrupt.to_string())?,
            None => Settings::default(),
        };
        s.passphrase = store.passphrase();
        s.keys = Some(keys);
        s.contacts = contacts;
        s.store = Some(store);
        s.locked = false;
        Ok(())
    }

    fn with<T>(&self, f: impl FnOnce(&mut Inner) -> Result<T, String>) -> Result<T, String> {
        let mut g = self.inner.lock().map_err(|_| "Interní chyba (zámek stavu)".to_string())?;
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
        self.with(|s| {
            if s.has_profile {
                return Err("Identita už existuje".into());
            }
            let keys = UserKeys::generate(now());
            let addr = address_uri(&keys.pk());
            if self.dir.is_some() {
                let device = self.device_key()?;
                let store = Store::create(&self.profile_path()?, &device as &dyn DeviceKey, passphrase.as_deref(), self.kdf).map_err(|e| e.to_string())?;
                store.put(Table::Identity, IDENTITY_KEY, zeroize::Zeroizing::new(keys.to_json()).as_bytes()).map_err(|e| e.to_string())?;
                s.store = Some(store);
            } else if passphrase.is_some() {
                return Err("Režim s heslem vyžaduje trvalý profil".into());
            }
            s.passphrase = passphrase.is_some();
            s.keys = Some(keys);
            s.has_profile = true;
            s.locked = false;
            Ok(addr)
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

    pub fn contact_add(&self, address: &str, name: Option<String>) -> Result<Contact, String> {
        let pk = parse_address(address.trim()).ok_or_else(|| "Neplatná adresa".to_string())?;
        self.with(|s| {
            let own = ready_keys(s)?.pk();
            if pk == own {
                return Err("To je vaše vlastní adresa".into());
            }
            if s.contacts.iter().any(|c| c.pk == pk) {
                return Err("Kontakt už existuje".into());
            }
            let name = name.map(|n| n.trim().to_string()).filter(|n| !n.is_empty()).unwrap_or_else(|| format!("Kontakt {}", s.contacts.len() + 1));
            let contact = Contact { id: hex_encode(&pk[..8]), name, address: address_uri(&pk), verified: false };
            save_contact(s, &pk, &contact)?;
            s.contacts.push(StoredContact { pk, contact: contact.clone() });
            Ok(contact)
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
            let (pk, contact) = (c.pk, c.contact.clone());
            save_contact(s, &pk, &contact)
        })
    }

    pub fn conversation_list(&self) -> Result<Vec<ConversationSummary>, String> {
        self.with(|s| ready_keys(s).map(|_| Vec::new()))
    }

    pub fn conversation_open(&self) -> Result<ConversationPage, String> {
        self.with(|s| ready_keys(s).map(|_| ConversationPage { messages: Vec::new(), more: false }))
    }

    pub fn message_send(&self) -> Result<Message, String> {
        Err(NOT_YET.into())
    }

    /// Smaže stav ratchetu a zprávy konverzace. Klíč záznamů je `id` (zprávy `id/ts/…`).
    pub fn conversation_delete(&self, id: &str) -> Result<(), String> {
        self.with(|s| {
            ready_keys(s)?;
            if let Some(store) = &s.store {
                let err = |e: StoreError| e.to_string();
                store.delete(Table::Conversations, id.as_bytes()).map_err(err)?;
                store.delete_prefix(Table::Messages, format!("{id}/").as_bytes()).map_err(err)?;
            }
            Ok(())
        })
    }

    pub fn connection_state(&self) -> ConnectionState {
        ConnectionState::Offline
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

    pub fn lock(&self) -> Result<(), String> {
        self.with(|s| {
            if s.has_profile {
                s.locked = true;
                if s.passphrase {
                    // Režim s heslem: klíče i master_key zmizí z paměti, odemkne jen heslo.
                    s.keys = None;
                    s.contacts.clear();
                    s.store = None;
                }
            }
            Ok(())
        })
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
        })
    }
}

fn save_contact(s: &Inner, pk: &[u8; 32], contact: &Contact) -> Result<(), String> {
    let Some(store) = &s.store else { return Ok(()) };
    let rec = serde_json::to_vec(&ContactRec { name: contact.name.clone(), verified: contact.verified }).map_err(|e| e.to_string())?;
    store.put(Table::Contacts, pk, &rec).map_err(|e| e.to_string())
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
    fn messaging_is_not_available_before_d3() {
        assert!(ready().message_send().is_err());
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
}
