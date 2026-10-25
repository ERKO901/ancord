//! Mailbox a E2E ratchet mezi klienty (`spec/PROTOCOL.md` §13.1 a §13.2). Node ho nikdy nepočítá, sdílí ho s ním jen klient.

use std::collections::BTreeMap;

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use hkdf::Hkdf;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::encoding::{b64u_decode, b64u_encode};

/// Kolik klíčů přeskočených zpráv se najednou uloží; větší mezera je desync.
pub const MAX_SKIP: u32 = 200;
const TAG_LEN: usize = 16;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum RatchetError {
    #[error("malformed ciphertext")]
    Malformed,
    #[error("message key already used or expired")]
    Replayed,
    #[error("ratchet gap too large")]
    Desync,
    #[error("authentication failed")]
    Auth,
    #[error("counter exhausted")]
    Exhausted,
}

/// Směr konverzace: do každého mailboxu píše jediný odesílatel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    #[serde(rename = "i2r")]
    InitiatorToResponder,
    #[serde(rename = "r2i")]
    ResponderToInitiator,
}

impl Direction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InitiatorToResponder => "i2r",
            Self::ResponderToInitiator => "r2i",
        }
    }

    pub fn opposite(self) -> Self {
        match self {
            Self::InitiatorToResponder => Self::ResponderToInitiator,
            Self::ResponderToInitiator => Self::InitiatorToResponder,
        }
    }
}

/// `sha256("ancord/v1/mailbox" ‖ 0x00 ‖ ss ‖ 0x00 ‖ dir)`.
pub fn mailbox_id(shared_secret: &[u8; 32], dir: Direction) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"ancord/v1/mailbox");
    h.update([0]);
    h.update(shared_secret);
    h.update([0]);
    h.update(dir.as_str().as_bytes());
    h.finalize().into()
}

fn expand<const N: usize>(ikm: &[u8], salt: Option<&[u8]>, info: &[u8]) -> [u8; N] {
    let mut out = [0u8; N];
    // `expand` selže jen při délce > 255 · HashLen; N je konstanta ≤ 44
    let _ = Hkdf::<Sha256>::new(salt, ikm).expand(info, &mut out);
    out
}

fn initial_chain_key(shared_secret: &[u8; 32], dir: Direction) -> [u8; 32] {
    let mut info = b"chain\0".to_vec();
    info.extend_from_slice(dir.as_str().as_bytes());
    expand(shared_secret, Some(b"ancord/v1/ratchet"), &info)
}

struct MessageKey {
    key: [u8; 32],
    nonce: [u8; 12],
}

fn message_key(chain_key: &[u8; 32]) -> MessageKey {
    let mk: [u8; 44] = expand(chain_key, None, b"ancord/v1/ratchet/msg");
    let mut key = [0u8; 32];
    let mut nonce = [0u8; 12];
    key.copy_from_slice(&mk[..32]);
    nonce.copy_from_slice(&mk[32..]);
    MessageKey { key, nonce }
}

fn next_chain_key(chain_key: &[u8; 32]) -> [u8; 32] {
    expand(chain_key, None, b"ancord/v1/ratchet/chain")
}

fn aad(mailbox_id: &[u8; 32], id: &[u8; 16], n: u32) -> Vec<u8> {
    let mut v = b"ancord/v1/msg\0".to_vec();
    v.extend_from_slice(mailbox_id);
    v.extend_from_slice(id);
    v.extend_from_slice(&n.to_be_bytes());
    v
}

/// Odesílací strana jednoho mailboxu. Stav se serializuje, aby klient přežil restart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sender {
    mailbox_id: [u8; 32],
    chain_key: [u8; 32],
    next_n: u32,
}

impl Sender {
    pub fn new(shared_secret: &[u8; 32], dir: Direction) -> Self {
        Self { mailbox_id: mailbox_id(shared_secret, dir), chain_key: initial_chain_key(shared_secret, dir), next_n: 0 }
    }

    pub fn mailbox_id(&self) -> [u8; 32] {
        self.mailbox_id
    }

    /// Zašifruje zprávu `id` a posune řetěz. Vrací `ct` (b64u) pro pole `ct` zprávy `send`.
    pub fn seal(&mut self, id: &[u8; 16], plaintext: &[u8]) -> Result<String, RatchetError> {
        let n = self.next_n;
        let next_n = n.checked_add(1).ok_or(RatchetError::Exhausted)?;
        let mk = message_key(&self.chain_key);
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&mk.key));
        let body = cipher
            .encrypt(Nonce::from_slice(&mk.nonce), Payload { msg: plaintext, aad: &aad(&self.mailbox_id, id, n) })
            .map_err(|_| RatchetError::Auth)?;
        self.chain_key = next_chain_key(&self.chain_key);
        self.next_n = next_n;
        let mut wire = n.to_be_bytes().to_vec();
        wire.extend_from_slice(&body);
        Ok(b64u_encode(&wire))
    }
}

/// Přijímací strana jednoho mailboxu.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receiver {
    mailbox_id: [u8; 32],
    chain_key: [u8; 32],
    next_n: u32,
    skipped: BTreeMap<u32, [u8; 32]>,
}

impl Receiver {
    pub fn new(shared_secret: &[u8; 32], dir: Direction) -> Self {
        Self {
            mailbox_id: mailbox_id(shared_secret, dir),
            chain_key: initial_chain_key(shared_secret, dir),
            next_n: 0,
            skipped: BTreeMap::new(),
        }
    }

    pub fn mailbox_id(&self) -> [u8; 32] {
        self.mailbox_id
    }

    /// Dešifruje zprávu. Stav se změní až po ověření tagu, takže podvržená zpráva řetěz neposune.
    pub fn open(&mut self, id: &[u8; 16], ct: &str) -> Result<Vec<u8>, RatchetError> {
        self.open_numbered(id, ct).map(|(_, plaintext)| plaintext)
    }

    /// Jako `open`, ale vrací i pořadové číslo zprávy. Podle něj klient řadí zprávy: `ts` má jen sekundovou
    /// přesnost, takže zprávy z téže sekundy server vrací v libovolném pořadí.
    pub fn open_numbered(&mut self, id: &[u8; 16], ct: &str) -> Result<(u32, Vec<u8>), RatchetError> {
        let wire = b64u_decode(ct).ok_or(RatchetError::Malformed)?;
        if wire.len() < 4 + TAG_LEN {
            return Err(RatchetError::Malformed);
        }
        let (counter, body) = wire.split_at(4);
        let n = u32::from_be_bytes(counter.try_into().map_err(|_| RatchetError::Malformed)?);
        let aad = aad(&self.mailbox_id, id, n);

        // klíč zprávy a změna stavu, která nastane až po ověření tagu
        let (msg_chain_key, advance) = if n < self.next_n {
            (*self.skipped.get(&n).ok_or(RatchetError::Replayed)?, None)
        } else {
            if n - self.next_n > MAX_SKIP {
                return Err(RatchetError::Desync);
            }
            let next_n = n.checked_add(1).ok_or(RatchetError::Exhausted)?;
            let mut ck = self.chain_key;
            let mut skipped = Vec::new();
            for i in self.next_n..n {
                skipped.push((i, ck));
                ck = next_chain_key(&ck);
            }
            (ck, Some((skipped, next_n)))
        };
        let mk = message_key(&msg_chain_key);
        let plaintext = ChaCha20Poly1305::new(Key::from_slice(&mk.key))
            .decrypt(Nonce::from_slice(&mk.nonce), Payload { msg: body, aad: &aad })
            .map_err(|_| RatchetError::Auth)?;

        match advance {
            None => {
                self.skipped.remove(&n);
            }
            Some((skipped, next_n)) => {
                self.chain_key = next_chain_key(&msg_chain_key);
                self.next_n = next_n;
                self.skipped.extend(skipped);
                while self.skipped.len() > MAX_SKIP as usize {
                    self.skipped.pop_first();
                }
            }
        }
        Ok((n, plaintext))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SS: [u8; 32] = [9; 32];

    fn pair() -> (Sender, Receiver) {
        (Sender::new(&SS, Direction::InitiatorToResponder), Receiver::new(&SS, Direction::InitiatorToResponder))
    }

    #[test]
    fn roundtrip_in_order() {
        let (mut tx, mut rx) = pair();
        for i in 0u8..5 {
            let id = [i; 16];
            let ct = tx.seal(&id, &[i, i, i]).unwrap();
            assert_eq!(rx.open(&id, &ct).unwrap(), vec![i, i, i]);
        }
    }

    #[test]
    fn out_of_order_and_replay() {
        let (mut tx, mut rx) = pair();
        let cts: Vec<_> = (0u8..4).map(|i| ([i; 16], tx.seal(&[i; 16], &[i]).unwrap())).collect();
        assert_eq!(rx.open(&cts[2].0, &cts[2].1).unwrap(), vec![2]);
        assert_eq!(rx.open(&cts[0].0, &cts[0].1).unwrap(), vec![0]);
        assert_eq!(rx.open(&cts[3].0, &cts[3].1).unwrap(), vec![3]);
        assert_eq!(rx.open(&cts[1].0, &cts[1].1).unwrap(), vec![1]);
        assert_eq!(rx.open(&cts[1].0, &cts[1].1), Err(RatchetError::Replayed));
        assert_eq!(rx.open(&cts[3].0, &cts[3].1), Err(RatchetError::Replayed));
    }

    #[test]
    fn forged_message_does_not_advance_state() {
        let (mut tx, mut rx) = pair();
        let id = [1; 16];
        let ct = tx.seal(&id, b"real").unwrap();
        // jiné id = jiné AAD
        assert_eq!(rx.open(&[2; 16], &ct), Err(RatchetError::Auth));
        assert_eq!(rx.open(&id, &ct).unwrap(), b"real");
    }

    #[test]
    fn wrong_direction_or_secret_fails() {
        let mut tx = Sender::new(&SS, Direction::InitiatorToResponder);
        let id = [3; 16];
        let ct = tx.seal(&id, b"x").unwrap();
        let mut other_dir = Receiver::new(&SS, Direction::ResponderToInitiator);
        assert_eq!(other_dir.open(&id, &ct), Err(RatchetError::Auth));
        let mut other_secret = Receiver::new(&[1; 32], Direction::InitiatorToResponder);
        assert_eq!(other_secret.open(&id, &ct), Err(RatchetError::Auth));
    }

    #[test]
    fn gap_limit() {
        let (mut tx, mut rx) = pair();
        let mut last = ([0; 16], String::new());
        for i in 0..=(MAX_SKIP + 1) {
            let id = [(i % 251) as u8; 16];
            last = (id, tx.seal(&id, b"m").unwrap());
        }
        assert_eq!(rx.open(&last.0, &last.1), Err(RatchetError::Desync));
    }

    #[test]
    fn garbage_is_rejected() {
        let (_, mut rx) = pair();
        assert_eq!(rx.open(&[0; 16], "!!"), Err(RatchetError::Malformed));
        assert_eq!(rx.open(&[0; 16], "AAAA"), Err(RatchetError::Malformed));
    }

    #[test]
    fn state_survives_serialization() {
        let (mut tx, mut rx) = pair();
        let id = [4; 16];
        let ct = tx.seal(&id, b"a").unwrap();
        let mut tx2: Sender = serde_json::from_str(&serde_json::to_string(&tx).unwrap()).unwrap();
        let mut rx2: Receiver = serde_json::from_str(&serde_json::to_string(&rx).unwrap()).unwrap();
        assert_eq!(rx2.open(&id, &ct).unwrap(), b"a");
        let id2 = [5; 16];
        let ct2 = tx2.seal(&id2, b"b").unwrap();
        assert_eq!(rx.open(&id, &ct).unwrap(), b"a");
        assert_eq!(rx.open(&id2, &ct2).unwrap(), b"b");
    }
}
