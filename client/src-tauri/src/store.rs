//! Šifrované úložiště klienta (§13.6, milník D2): jeden soubor redb, hodnoty šifrované po záznamech.
//!
//! Hodnota je XChaCha20-Poly1305, klíč tabulky `HKDF(master_key, "ancord/v1/client/" ‖ tabulka)`,
//! AAD = tabulka ‖ klíč záznamu (záznam nejde přesunout do jiné tabulky ani pod jiný klíč).
//! `master_key` (32 B náhodných) je v souboru jen zabalený: klíčem zařízení (`DeviceKey`) a volitelně
//! i klíčem z hesla (Argon2id). Klíč zařízení je za rozhraním, protože platformní úložiště
//! (Keychain, DPAPI, Secret Service, Android Keystore) se připojují zvlášť; zatím je jen souborový.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use hkdf::Hkdf;
use rand::RngCore;
use redb::{Database, TableDefinition};
use sha2::Sha256;
use std::sync::atomic::{AtomicBool, Ordering};

use zeroize::Zeroizing;

const CONTEXT: &str = "ancord/v1/client/";
const WRAP_VERSION: u8 = 1;
const MODE_DEVICE: u8 = 0;
const MODE_PASSPHRASE: u8 = 1;
const NONCE_LEN: usize = 24;
const SALT_LEN: usize = 16;
/// version, mode, m_kib (4), t (4), salt, nonce.
const WRAP_HEADER: usize = 2 + 4 + 4 + SALT_LEN + NONCE_LEN;

/// Tabulky šifrovaných záznamů. Pevný seznam: neznámé jméno je chyba programu, ne dat.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Table {
    Identity,
    Conversations,
    Messages,
    Settings,
    Contacts,
}

impl Table {
    fn name(self) -> &'static str {
        match self {
            Table::Identity => "identity",
            Table::Conversations => "conversations",
            Table::Messages => "messages",
            Table::Settings => "settings",
            Table::Contacts => "contacts",
        }
    }

    fn def(self) -> TableDefinition<'static, &'static [u8], &'static [u8]> {
        TableDefinition::new(self.name())
    }
}

const META: TableDefinition<&str, &[u8]> = TableDefinition::new("meta");
const WRAP_KEY: &str = "wrap";

#[derive(Debug, PartialEq, Eq)]
pub enum StoreError {
    /// Špatné heslo nebo cizí klíč zařízení.
    Unlock,
    /// Heslo je potřeba (nebo naopak není), ale volající ho nedodal.
    Passphrase,
    /// Záznam se nepodařilo ověřit: poškozený, podvržený nebo přesunutý.
    Corrupt,
    Io(String),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::Unlock => write!(f, "Nepodařilo se odemknout úložiště (špatné heslo?)"),
            StoreError::Passphrase => write!(f, "Úložiště vyžaduje heslo"),
            StoreError::Corrupt => write!(f, "Záznam v úložišti je poškozený"),
            StoreError::Io(e) => write!(f, "Úložiště: {e}"),
        }
    }
}

fn io<E: fmt::Display>(e: E) -> StoreError {
    StoreError::Io(e.to_string())
}

/// Klíč zařízení: druhý faktor, který se u hesla nevyvozuje z uživatele.
pub trait DeviceKey {
    fn load_or_create(&self) -> Result<Zeroizing<[u8; 32]>, StoreError>;
}

/// Klíč v souboru vedle úložiště (oprávnění 0600). Chrání před zkopírováním samotného souboru
/// profilu, ne před útočníkem s přístupem k celé složce; k tomu slouží platformní úložiště
/// klíčů, které se přidá za stejné rozhraní.
pub struct FileDeviceKey(pub PathBuf);

impl DeviceKey for FileDeviceKey {
    fn load_or_create(&self) -> Result<Zeroizing<[u8; 32]>, StoreError> {
        match fs::read(&self.0) {
            Ok(bytes) => {
                let bytes = Zeroizing::new(bytes);
                let key: [u8; 32] = bytes.as_slice().try_into().map_err(|_| StoreError::Unlock)?;
                Ok(Zeroizing::new(key))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let mut key = Zeroizing::new([0u8; 32]);
                rand::rngs::OsRng.fill_bytes(key.as_mut());
                write_private(&self.0, key.as_ref())?;
                Ok(key)
            }
            Err(e) => Err(io(e)),
        }
    }
}

fn write_private(path: &Path, data: &[u8]) -> Result<(), StoreError> {
    use std::io::Write;
    let mut opts = fs::OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut opts, 0o600);
    let mut f = opts.open(path).map_err(io)?;
    f.write_all(data).map_err(io)?;
    f.sync_all().map_err(io)
}

/// Klíč zařízení v platformním úložišti (Keychain, DPAPI/Credential Manager, Secret Service).
#[cfg(not(target_os = "android"))]
pub struct KeychainDeviceKey {
    pub service: String,
    pub user: String,
}

#[cfg(not(target_os = "android"))]
impl DeviceKey for KeychainDeviceKey {
    fn load_or_create(&self) -> Result<Zeroizing<[u8; 32]>, StoreError> {
        let entry = keyring::Entry::new(&self.service, &self.user).map_err(io)?;
        match entry.get_secret() {
            Ok(bytes) => {
                let bytes = Zeroizing::new(bytes);
                let key: [u8; 32] = bytes.as_slice().try_into().map_err(|_| StoreError::Unlock)?;
                Ok(Zeroizing::new(key))
            }
            Err(keyring::Error::NoEntry) => {
                let mut key = Zeroizing::new([0u8; 32]);
                rand::rngs::OsRng.fill_bytes(key.as_mut());
                entry.set_secret(key.as_ref()).map_err(io)?;
                Ok(key)
            }
            Err(e) => Err(io(e)),
        }
    }
}

/// Platformní úložiště, kde existuje; jinak souborový klíč. Existující `device.key` má přednost, aby profil
/// vytvořený bez keychainu (stroj bez Secret Service) šel otevřít i poté, co se keychain objeví.
/// Profil s keychainem se bez něj neotevře (chyba `Io`), nikdy se tiše nevytvoří nový klíč.
pub struct AutoDeviceKey {
    pub file: FileDeviceKey,
    pub keychain: Option<Box<dyn DeviceKey + Send + Sync>>,
}

impl DeviceKey for AutoDeviceKey {
    fn load_or_create(&self) -> Result<Zeroizing<[u8; 32]>, StoreError> {
        if self.file.0.exists() {
            return self.file.load_or_create();
        }
        match &self.keychain {
            Some(k) => match k.load_or_create() {
                Ok(key) => Ok(key),
                // jen při zakládání profilu: keychain nejde použít, profil dostane souborový klíč
                Err(StoreError::Io(_)) if !self.profile_exists() => self.file.load_or_create(),
                Err(e) => Err(e),
            },
            None => self.file.load_or_create(),
        }
    }
}

impl AutoDeviceKey {
    fn profile_exists(&self) -> bool {
        self.file.0.with_file_name("profile.redb").exists()
    }
}

/// Cena Argon2id. Uložená v hlavičce zabaleného klíče, takže ji lze později zvýšit.
#[derive(Clone, Copy, Debug)]
pub struct KdfParams {
    pub m_kib: u32,
    pub t: u32,
}

impl Default for KdfParams {
    fn default() -> Self {
        Self { m_kib: 64 * 1024, t: 3 }
    }
}

/// Klíč záznamu a dešifrovaná hodnota.
pub type Record = (Vec<u8>, Zeroizing<Vec<u8>>);

pub struct Store {
    db: Database,
    master: Zeroizing<[u8; 32]>,
    passphrase: AtomicBool,
}

impl Store {
    pub fn exists(path: &Path) -> bool {
        path.is_file()
    }

    /// Je úložiště chráněné heslem? Čte jen nešifrovanou hlavičku, takže jde zavolat před odemčením.
    pub fn uses_passphrase(path: &Path) -> Result<bool, StoreError> {
        let db = Database::open(path).map_err(io)?;
        let blob = read_wrap(&db)?;
        Ok(blob.get(1) == Some(&MODE_PASSPHRASE))
    }

    pub fn create(path: &Path, device: &dyn DeviceKey, passphrase: Option<&str>, kdf: KdfParams) -> Result<Self, StoreError> {
        if path.exists() {
            return Err(StoreError::Io("úložiště už existuje".into()));
        }
        let passphrase = passphrase.filter(|p| !p.is_empty());
        let mut master = Zeroizing::new([0u8; 32]);
        rand::rngs::OsRng.fill_bytes(master.as_mut());
        let device_key = device.load_or_create()?;
        let blob = wrap(&master, &device_key, passphrase, kdf)?;
        let db = Database::create(path).map_err(io)?;
        let tx = db.begin_write().map_err(io)?;
        {
            let mut meta = tx.open_table(META).map_err(io)?;
            meta.insert(WRAP_KEY, blob.as_slice()).map_err(io)?;
            for t in [Table::Identity, Table::Conversations, Table::Messages, Table::Settings, Table::Contacts] {
                tx.open_table(t.def()).map_err(io)?;
            }
        }
        tx.commit().map_err(io)?;
        Ok(Self { db, master, passphrase: AtomicBool::new(passphrase.is_some()) })
    }

    pub fn open(path: &Path, device: &dyn DeviceKey, passphrase: Option<&str>) -> Result<Self, StoreError> {
        let db = Database::open(path).map_err(io)?;
        let blob = read_wrap(&db)?;
        let needs = blob.get(1) == Some(&MODE_PASSPHRASE);
        let passphrase = passphrase.filter(|p| !p.is_empty());
        if needs && passphrase.is_none() {
            return Err(StoreError::Passphrase);
        }
        let device_key = device.load_or_create()?;
        let master = unwrap_key(&blob, &device_key, passphrase.filter(|_| needs))?;
        Ok(Self { db, master, passphrase: AtomicBool::new(needs) })
    }

    pub fn passphrase(&self) -> bool {
        self.passphrase.load(Ordering::Relaxed)
    }

    /// Změní (nebo nově nastaví) heslo: hlavní klíč se zabalí znovu, data v tabulkách ani biometrický obal zůstanou.
    /// Staré heslo se ověří, i když se nemění pro jeho vlastní účel (u profilu s heslem je povinné).
    pub fn change_passphrase(&self, device: &dyn DeviceKey, old: Option<&str>, new: &str, kdf: KdfParams) -> Result<(), StoreError> {
        if new.is_empty() {
            return Err(StoreError::Passphrase);
        }
        let blob = read_wrap(&self.db)?;
        let needs = blob.get(1) == Some(&MODE_PASSPHRASE);
        let old = old.filter(|p| !p.is_empty());
        if needs && old.is_none() {
            return Err(StoreError::Passphrase);
        }
        let device_key = device.load_or_create()?;
        let master = unwrap_key(&blob, &device_key, old.filter(|_| needs))?;
        if master.as_ref() != self.master.as_ref() {
            return Err(StoreError::Unlock);
        }
        let fresh = wrap(&self.master, &device_key, Some(new), kdf)?;
        let tx = self.db.begin_write().map_err(io)?;
        {
            let mut meta = tx.open_table(META).map_err(io)?;
            meta.insert(WRAP_KEY, fresh.as_slice()).map_err(io)?;
        }
        tx.commit().map_err(io)?;
        self.passphrase.store(true, Ordering::Relaxed);
        Ok(())
    }

    /// Hlavní klíč úložiště pro obálku biometrikou (jen uvnitř shellu, do UI nikdy).
    pub fn master_key(&self) -> Zeroizing<[u8; 32]> {
        self.master.clone()
    }

    /// Otevře úložiště hlavním klíčem (biometrické odemčení). Špatný klíč se projeví až při čtení záznamů (`Corrupt`).
    pub fn open_with_master(path: &Path, master: &[u8]) -> Result<Self, StoreError> {
        let db = Database::open(path).map_err(io)?;
        let blob = read_wrap(&db)?;
        let master: [u8; 32] = master.try_into().map_err(|_| StoreError::Unlock)?;
        Ok(Self { db, master: Zeroizing::new(master), passphrase: AtomicBool::new(blob.get(1) == Some(&MODE_PASSPHRASE)) })
    }

    fn cipher(&self, table: Table) -> Result<XChaCha20Poly1305, StoreError> {
        let hk = Hkdf::<Sha256>::new(None, self.master.as_ref());
        let mut key = Zeroizing::new([0u8; 32]);
        hk.expand(format!("{CONTEXT}{}", table.name()).as_bytes(), key.as_mut()).map_err(io)?;
        Ok(XChaCha20Poly1305::new(key.as_ref().into()))
    }

    fn aad(table: Table, key: &[u8]) -> Vec<u8> {
        let mut aad = table.name().as_bytes().to_vec();
        aad.extend_from_slice(key);
        aad
    }

    fn seal(&self, table: Table, key: &[u8], value: &[u8]) -> Result<Vec<u8>, StoreError> {
        let mut nonce = [0u8; NONCE_LEN];
        rand::rngs::OsRng.fill_bytes(&mut nonce);
        let ct = self
            .cipher(table)?
            .encrypt(XNonce::from_slice(&nonce), Payload { msg: value, aad: &Self::aad(table, key) })
            .map_err(|_| StoreError::Corrupt)?;
        let mut out = nonce.to_vec();
        out.extend_from_slice(&ct);
        Ok(out)
    }

    fn open_record(&self, table: Table, key: &[u8], stored: &[u8]) -> Result<Zeroizing<Vec<u8>>, StoreError> {
        if stored.len() < NONCE_LEN {
            return Err(StoreError::Corrupt);
        }
        let (nonce, ct) = stored.split_at(NONCE_LEN);
        let pt = self
            .cipher(table)?
            .decrypt(XNonce::from_slice(nonce), Payload { msg: ct, aad: &Self::aad(table, key) })
            .map_err(|_| StoreError::Corrupt)?;
        Ok(Zeroizing::new(pt))
    }

    pub fn put(&self, table: Table, key: &[u8], value: &[u8]) -> Result<(), StoreError> {
        let sealed = self.seal(table, key, value)?;
        let tx = self.db.begin_write().map_err(io)?;
        {
            let mut t = tx.open_table(table.def()).map_err(io)?;
            t.insert(key, sealed.as_slice()).map_err(io)?;
        }
        tx.commit().map_err(io)
    }

    pub fn get(&self, table: Table, key: &[u8]) -> Result<Option<Zeroizing<Vec<u8>>>, StoreError> {
        let tx = self.db.begin_read().map_err(io)?;
        let t = tx.open_table(table.def()).map_err(io)?;
        let Some(v) = t.get(key).map_err(io)? else { return Ok(None) };
        self.open_record(table, key, v.value()).map(Some)
    }

    #[cfg(test)]
    pub fn delete(&self, table: Table, key: &[u8]) -> Result<bool, StoreError> {
        let tx = self.db.begin_write().map_err(io)?;
        let removed = {
            let mut t = tx.open_table(table.def()).map_err(io)?;
            let removed = t.remove(key).map_err(io)?.is_some();
            removed
        };
        tx.commit().map_err(io)?;
        Ok(removed)
    }

    /// Všechny záznamy, jejichž klíč začíná `prefix`, ve vzestupném pořadí klíčů.
    pub fn scan(&self, table: Table, prefix: &[u8]) -> Result<Vec<Record>, StoreError> {
        let tx = self.db.begin_read().map_err(io)?;
        let t = tx.open_table(table.def()).map_err(io)?;
        let mut out = Vec::new();
        for row in t.range(prefix..).map_err(io)? {
            let (k, v) = row.map_err(io)?;
            if !k.value().starts_with(prefix) {
                break;
            }
            out.push((k.value().to_vec(), self.open_record(table, k.value(), v.value())?));
        }
        Ok(out)
    }

    /// Nejvýše `limit` záznamů s prefixem `prefix`, jejichž klíč je menší než `prefix ‖ before` (bez `before`
    /// všechny), od nejnovějšího (sestupně). Druhá hodnota říká, zda zbývají další starší.
    pub fn scan_rev(&self, table: Table, prefix: &[u8], before: Option<&[u8]>, limit: usize) -> Result<(Vec<Record>, bool), StoreError> {
        let end = match before {
            Some(b) => [prefix, b].concat(),
            None => prefix_end(prefix),
        };
        let tx = self.db.begin_read().map_err(io)?;
        let t = tx.open_table(table.def()).map_err(io)?;
        let mut out = Vec::new();
        for row in t.range(prefix..end.as_slice()).map_err(io)?.rev() {
            let (k, v) = row.map_err(io)?;
            if out.len() == limit {
                return Ok((out, true));
            }
            out.push((k.value().to_vec(), self.open_record(table, k.value(), v.value())?));
        }
        Ok((out, false))
    }

    /// Smaže všechny záznamy s daným prefixem (`conversation_delete`).
    pub fn delete_prefix(&self, table: Table, prefix: &[u8]) -> Result<usize, StoreError> {
        let keys: Vec<Vec<u8>> = {
            let tx = self.db.begin_read().map_err(io)?;
            let t = tx.open_table(table.def()).map_err(io)?;
            let mut keys = Vec::new();
            for row in t.range(prefix..).map_err(io)? {
                let (k, _) = row.map_err(io)?;
                if !k.value().starts_with(prefix) {
                    break;
                }
                keys.push(k.value().to_vec());
            }
            keys
        };
        let tx = self.db.begin_write().map_err(io)?;
        {
            let mut t = tx.open_table(table.def()).map_err(io)?;
            for k in &keys {
                t.remove(k.as_slice()).map_err(io)?;
            }
        }
        tx.commit().map_err(io)?;
        Ok(keys.len())
    }
}

/// Nejmenší klíč větší než všechny klíče s prefixem `prefix` (prefix nesmí být prázdný ani samé `0xff`).
fn prefix_end(prefix: &[u8]) -> Vec<u8> {
    let mut end = prefix.to_vec();
    while let Some(last) = end.pop() {
        if last < u8::MAX {
            end.push(last + 1);
            return end;
        }
    }
    vec![u8::MAX; prefix.len() + 1]
}

fn read_wrap(db: &Database) -> Result<Vec<u8>, StoreError> {
    let tx = db.begin_read().map_err(io)?;
    let meta = tx.open_table(META).map_err(io)?;
    let blob = meta.get(WRAP_KEY).map_err(io)?.ok_or(StoreError::Corrupt)?;
    Ok(blob.value().to_vec())
}

fn wrap_key(device: &[u8; 32], passphrase: Option<&str>, salt: &[u8], kdf: KdfParams) -> Result<Zeroizing<[u8; 32]>, StoreError> {
    let mut ikm = Zeroizing::new(device.to_vec());
    if let Some(p) = passphrase {
        let params = Params::new(kdf.m_kib, kdf.t, 1, Some(32)).map_err(io)?;
        let mut derived = Zeroizing::new([0u8; 32]);
        Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
            .hash_password_into(p.as_bytes(), salt, derived.as_mut())
            .map_err(io)?;
        ikm.extend_from_slice(derived.as_ref());
    }
    let mut out = Zeroizing::new([0u8; 32]);
    Hkdf::<Sha256>::new(Some(salt), &ikm).expand(format!("{CONTEXT}wrap").as_bytes(), out.as_mut()).map_err(io)?;
    Ok(out)
}

fn wrap(master: &[u8; 32], device: &[u8; 32], passphrase: Option<&str>, kdf: KdfParams) -> Result<Vec<u8>, StoreError> {
    let mode = if passphrase.is_some() { MODE_PASSPHRASE } else { MODE_DEVICE };
    let mut salt = [0u8; SALT_LEN];
    let mut nonce = [0u8; NONCE_LEN];
    rand::rngs::OsRng.fill_bytes(&mut salt);
    rand::rngs::OsRng.fill_bytes(&mut nonce);
    let mut header = vec![WRAP_VERSION, mode];
    header.extend_from_slice(&kdf.m_kib.to_be_bytes());
    header.extend_from_slice(&kdf.t.to_be_bytes());
    header.extend_from_slice(&salt);
    header.extend_from_slice(&nonce);
    let key = wrap_key(device, passphrase, &salt, kdf)?;
    let ct = XChaCha20Poly1305::new(key.as_ref().into())
        .encrypt(XNonce::from_slice(&nonce), Payload { msg: master, aad: &header })
        .map_err(|_| StoreError::Corrupt)?;
    header.extend_from_slice(&ct);
    Ok(header)
}

fn unwrap_key(blob: &[u8], device: &[u8; 32], passphrase: Option<&str>) -> Result<Zeroizing<[u8; 32]>, StoreError> {
    if blob.len() < WRAP_HEADER || blob[0] != WRAP_VERSION {
        return Err(StoreError::Corrupt);
    }
    let kdf = KdfParams {
        m_kib: u32::from_be_bytes(blob[2..6].try_into().map_err(|_| StoreError::Corrupt)?),
        t: u32::from_be_bytes(blob[6..10].try_into().map_err(|_| StoreError::Corrupt)?),
    };
    let salt = &blob[10..10 + SALT_LEN];
    let nonce = &blob[10 + SALT_LEN..WRAP_HEADER];
    let key = wrap_key(device, passphrase, salt, kdf)?;
    let pt = XChaCha20Poly1305::new(key.as_ref().into())
        .decrypt(XNonce::from_slice(nonce), Payload { msg: &blob[WRAP_HEADER..], aad: &blob[..WRAP_HEADER] })
        .map_err(|_| StoreError::Unlock)?;
    let pt = Zeroizing::new(pt);
    let master: [u8; 32] = pt.as_slice().try_into().map_err(|_| StoreError::Corrupt)?;
    Ok(Zeroizing::new(master))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    const FAST: KdfParams = KdfParams { m_kib: 8, t: 1 };

    struct Env {
        _dir: tempfile::TempDir,
        db: PathBuf,
        device: FileDeviceKey,
    }

    fn env() -> Env {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("profile.redb");
        let device = FileDeviceKey(dir.path().join("device.key"));
        Env { _dir: dir, db, device }
    }

    #[test]
    fn records_survive_a_restart() {
        let e = env();
        let s = Store::create(&e.db, &e.device, None, FAST).unwrap();
        s.put(Table::Identity, b"me", b"tajny klic").unwrap();
        drop(s);
        let s = Store::open(&e.db, &e.device, None).unwrap();
        assert_eq!(s.get(Table::Identity, b"me").unwrap().unwrap().as_slice(), b"tajny klic");
        assert!(s.get(Table::Identity, b"jiny").unwrap().is_none());
    }

    #[test]
    fn plaintext_is_not_in_the_file() {
        let e = env();
        let s = Store::create(&e.db, &e.device, None, FAST).unwrap();
        s.put(Table::Messages, b"c1/0001", b"velmi citlivy text zpravy").unwrap();
        drop(s);
        let raw = fs::read(&e.db).unwrap();
        assert!(!raw.windows(11).any(|w| w == b"citlivy tex"));
    }

    #[test]
    fn tampered_or_moved_record_is_rejected() {
        let e = env();
        let s = Store::create(&e.db, &e.device, None, FAST).unwrap();
        s.put(Table::Messages, b"a", b"hodnota").unwrap();
        // Záznam přesunutý pod jiný klíč a do jiné tabulky se neověří (AAD).
        let sealed = {
            let tx = s.db.begin_read().unwrap();
            let t = tx.open_table(Table::Messages.def()).unwrap();
            let v = t.get(b"a".as_slice()).unwrap().unwrap().value().to_vec();
            v
        };
        let tx = s.db.begin_write().unwrap();
        {
            tx.open_table(Table::Messages.def()).unwrap().insert(b"b".as_slice(), sealed.as_slice()).unwrap();
            tx.open_table(Table::Settings.def()).unwrap().insert(b"a".as_slice(), sealed.as_slice()).unwrap();
            let mut flipped = sealed.clone();
            let last = flipped.len() - 1;
            flipped[last] ^= 1;
            tx.open_table(Table::Contacts.def()).unwrap().insert(b"a".as_slice(), flipped.as_slice()).unwrap();
        }
        tx.commit().unwrap();
        assert_eq!(s.get(Table::Messages, b"b").unwrap_err(), StoreError::Corrupt);
        assert_eq!(s.get(Table::Settings, b"a").unwrap_err(), StoreError::Corrupt);
        assert_eq!(s.get(Table::Contacts, b"a").unwrap_err(), StoreError::Corrupt);
        assert!(s.get(Table::Messages, b"a").is_ok());
    }

    #[test]
    fn foreign_device_key_does_not_open() {
        let e = env();
        Store::create(&e.db, &e.device, None, FAST).unwrap();
        let other = FileDeviceKey(e.db.with_file_name("other.key"));
        assert_eq!(Store::open(&e.db, &other, None).err(), Some(StoreError::Unlock));
    }

    #[test]
    fn passphrase_is_required_and_checked() {
        let e = env();
        let s = Store::create(&e.db, &e.device, Some("spravne heslo"), FAST).unwrap();
        s.put(Table::Settings, b"k", b"v").unwrap();
        drop(s);
        assert!(Store::uses_passphrase(&e.db).unwrap());
        assert_eq!(Store::open(&e.db, &e.device, None).err(), Some(StoreError::Passphrase));
        assert_eq!(Store::open(&e.db, &e.device, Some("spatne")).err(), Some(StoreError::Unlock));
        let s = Store::open(&e.db, &e.device, Some("spravne heslo")).unwrap();
        assert!(s.passphrase());
        assert_eq!(s.get(Table::Settings, b"k").unwrap().unwrap().as_slice(), b"v");
    }

    struct Fake(Result<[u8; 32], ()>);
    impl DeviceKey for Fake {
        fn load_or_create(&self) -> Result<Zeroizing<[u8; 32]>, StoreError> {
            self.0.map(Zeroizing::new).map_err(|()| StoreError::Io("keychain".into()))
        }
    }

    fn auto(e: &Env, k: Option<Result<[u8; 32], ()>>) -> AutoDeviceKey {
        AutoDeviceKey { file: FileDeviceKey(e.device.0.clone()), keychain: k.map(|r| Box::new(Fake(r)) as Box<dyn DeviceKey + Send + Sync>) }
    }

    #[test]
    fn keychain_is_preferred_and_file_is_not_created() {
        let e = env();
        let a = auto(&e, Some(Ok([7; 32])));
        Store::create(&e.db, &a, None, FAST).unwrap();
        assert!(!e.device.0.exists());
        Store::open(&e.db, &a, None).unwrap();
    }

    #[test]
    fn unavailable_keychain_falls_back_only_when_creating() {
        let e = env();
        let broken = auto(&e, Some(Err(())));
        Store::create(&e.db, &broken, None, FAST).unwrap();
        assert!(e.device.0.exists());
        // profil s keychainem se při výpadku keychainu neotevře a nevznikne nový klíč
        let e2 = env();
        Store::create(&e2.db, &auto(&e2, Some(Ok([1; 32]))), None, FAST).unwrap();
        assert!(Store::open(&e2.db, &auto(&e2, Some(Err(()))), None).is_err());
        assert!(!e2.device.0.exists());
    }

    #[test]
    fn device_mode_reports_no_passphrase() {
        let e = env();
        Store::create(&e.db, &e.device, None, FAST).unwrap();
        assert!(!Store::uses_passphrase(&e.db).unwrap());
    }

    #[test]
    fn scan_and_delete_prefix_are_scoped() {
        let e = env();
        let s = Store::create(&e.db, &e.device, None, FAST).unwrap();
        for k in ["c1/1", "c1/2", "c2/1"] {
            s.put(Table::Messages, k.as_bytes(), k.as_bytes()).unwrap();
        }
        let rows = s.scan(Table::Messages, b"c1/").unwrap();
        assert_eq!(rows.iter().map(|(k, _)| k.as_slice()).collect::<Vec<_>>(), vec![b"c1/1".as_slice(), b"c1/2".as_slice()]);
        assert_eq!(s.delete_prefix(Table::Messages, b"c1/").unwrap(), 2);
        assert_eq!(s.scan(Table::Messages, b"").unwrap().len(), 1);
        assert!(s.delete(Table::Messages, b"c2/1").unwrap());
        assert!(!s.delete(Table::Messages, b"c2/1").unwrap());
    }
}
