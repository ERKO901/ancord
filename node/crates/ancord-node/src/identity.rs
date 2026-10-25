//! Perzistentní identita nodu: Ed25519 klíč na disku, `node_id = base32(blake3(pubkey))` (`spec/PROTOCOL.md` §2).

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;

use crate::encoding::{b64u_decode_array, b64u_encode, base32_encode};
use crate::error::NodeError;

pub const KEY_FILE: &str = "identity.key";
pub const HOSTNAME_ID_CHARS: usize = 16;

pub struct Identity {
    key: SigningKey,
    node_id: String,
}

impl Identity {
    pub fn from_seed(seed: [u8; 32]) -> Self {
        let key = SigningKey::from_bytes(&seed);
        let node_id = node_id_of(key.verifying_key().as_bytes());
        Self { key, node_id }
    }

    /// Načte klíč z `dir/identity.key`, jinak vytvoří nový (práva 0600). Restart tak zachová `node_id`.
    pub fn load_or_create(dir: &Path) -> Result<Self, NodeError> {
        fs::create_dir_all(dir)?;
        let path: PathBuf = dir.join(KEY_FILE);
        match fs::read_to_string(&path) {
            Ok(text) => {
                let mode = fs::metadata(&path)?.permissions().mode() & 0o777;
                if mode & 0o077 != 0 {
                    return Err(NodeError(format!(
                        "{} má práva {mode:o}, privátní klíč musí být přístupný jen vlastníkovi (chmod 600)",
                        path.display()
                    )));
                }
                let seed = b64u_decode_array::<32>(text.trim())
                    .ok_or_else(|| NodeError(format!("{} neobsahuje 32B klíč v base64url", path.display())))?;
                Ok(Self::from_seed(seed))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let key = SigningKey::generate(&mut OsRng);
                let mut file = OpenOptions::new().write(true).create_new(true).mode(0o600).open(&path)?;
                file.write_all(b64u_encode(&key.to_bytes()).as_bytes())?;
                file.write_all(b"\n")?;
                file.sync_all()?;
                Ok(Self::from_seed(key.to_bytes()))
            }
            Err(e) => Err(e.into()),
        }
    }

    pub fn signing_key(&self) -> &SigningKey {
        &self.key
    }

    pub fn pubkey(&self) -> [u8; 32] {
        self.key.verifying_key().to_bytes()
    }

    pub fn pubkey_b64(&self) -> String {
        b64u_encode(&self.pubkey())
    }

    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    pub fn sign(&self, domain: &str, payload: &[u8]) -> [u8; 64] {
        crate::sign::sign(&self.key, domain, payload)
    }
}

pub fn node_id_of(pubkey: &[u8; 32]) -> String {
    base32_encode(blake3::hash(pubkey).as_bytes())
}

pub fn hostname_of(node_id: &str, zone: &str) -> String {
    let prefix: String = node_id.chars().take(HOSTNAME_ID_CHARS).collect();
    format!("node{prefix}.{zone}")
}

/// Hostname vrácený Workerem musí odpovídat našemu `node_id` (jinak by nám Worker mohl podstrčit cizí jméno).
pub fn hostname_matches(node_id: &str, hostname: &str) -> bool {
    let prefix: String = node_id.chars().take(HOSTNAME_ID_CHARS).collect();
    hostname
        .strip_prefix("node")
        .and_then(|rest| rest.strip_prefix(prefix.as_str()))
        .is_some_and(|rest| rest.starts_with('.') && rest.len() > 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_persists_with_private_mode() {
        let dir = tempfile::tempdir().unwrap();
        let a = Identity::load_or_create(dir.path()).unwrap();
        let b = Identity::load_or_create(dir.path()).unwrap();
        assert_eq!(a.node_id(), b.node_id());
        let mode = fs::metadata(dir.path().join(KEY_FILE)).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn loose_permissions_are_rejected() {
        let dir = tempfile::tempdir().unwrap();
        Identity::load_or_create(dir.path()).unwrap();
        let path = dir.path().join(KEY_FILE);
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(Identity::load_or_create(dir.path()).is_err());
    }

    #[test]
    fn hostname_check() {
        let id = "qnlbvwzzr7mh7dt63azrx7zpzokfom6mgajiphfz7kyhskdgobra";
        assert!(hostname_matches(id, "nodeqnlbvwzzr7mh7dt6.melonet.cz"));
        assert!(!hostname_matches(id, "nodeqnlbvwzzr7mh7dt7.melonet.cz"));
        assert!(!hostname_matches(id, "nodeqnlbvwzzr7mh7dt6."));
    }
}
