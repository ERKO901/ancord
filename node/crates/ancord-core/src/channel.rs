//! Zabezpečený kanál nad TLS Cloudflare (`spec/PROTOCOL.md` §14).
//!
//! Cloudflare ukončuje TLS na okraji, takže by jinak viděl `mailbox_id`, tickety a `push_id`. Proto klient s nodem
//! a nody mezi sebou vedou vlastní autentizovanou výměnu klíčů (X25519 + podpis Ed25519 identity nodu) a vše dál
//! posílají šifrované (ChaCha20-Poly1305, mezi nody XChaCha20-Poly1305) s paddingem na násobky 256 B.

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce, XChaCha20Poly1305, XNonce};
use hkdf::Hkdf;
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::{Digest, Sha256};
use x25519_dalek::{PublicKey, StaticSecret};
use zeroize::Zeroizing;

pub const PAD_BLOCK: usize = 256;
pub const TAG_LEN: usize = 16;
/// Nejvyšší počet rámců v jednom směru; pak je nutné navázat nový kanál (nonce se nesmí opakovat).
pub const MAX_FRAMES: u64 = 1 << 32;

const CHAN_LABEL: &[u8] = b"ancord/v1/chan";
const CHAN_KEYS_INFO: &[u8] = b"ancord/v1/chan/keys";
const CHAN_FRAME_AAD: &[u8] = b"ancord/v1/chan/frame";
const N2N_LABEL: &[u8] = b"ancord/v1/n2n";
const N2N_KEYS_INFO: &[u8] = b"ancord/v1/n2n/keys";
const N2N_REQ_AAD: &[u8] = b"ancord/v1/n2n/req";
const N2N_RES_AAD: &[u8] = b"ancord/v1/n2n/res";

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ChannelError {
    #[error("invalid public key")]
    BadPublicKey,
    #[error("frame could not be authenticated")]
    Decrypt,
    #[error("malformed padding")]
    Padding,
    #[error("frame counter exhausted")]
    Exhausted,
    #[error("message is too large")]
    TooLarge,
}

/// Efemerní klíč X25519. Soukromá část se při zahození vynuluje.
pub struct Ephemeral {
    secret: StaticSecret,
    public: [u8; 32],
}

impl Ephemeral {
    pub fn generate() -> Self {
        Self::from_bytes(Zeroizing::new(random_array::<32>()))
    }

    pub fn from_bytes(bytes: Zeroizing<[u8; 32]>) -> Self {
        let secret = StaticSecret::from(*bytes);
        let public = PublicKey::from(&secret).to_bytes();
        Self { secret, public }
    }

    pub fn public(&self) -> [u8; 32] {
        self.public
    }

    /// Sdílené tajemství. Body malého řádu (výstup samé nuly) se odmítají.
    pub fn diffie_hellman(self, peer: &[u8; 32]) -> Result<Zeroizing<[u8; 32]>, ChannelError> {
        let shared = self.secret.diffie_hellman(&PublicKey::from(*peer));
        if !shared.was_contributory() {
            return Err(ChannelError::BadPublicKey);
        }
        Ok(Zeroizing::new(shared.to_bytes()))
    }
}

pub fn random_array<const N: usize>() -> [u8; N] {
    let mut out = [0u8; N];
    OsRng.fill_bytes(&mut out);
    out
}

fn sha256_parts(parts: &[&[u8]]) -> [u8; 32] {
    let mut h = Sha256::new();
    for p in parts {
        h.update(p);
    }
    h.finalize().into()
}

fn derive(shared: &[u8; 32], th: &[u8; 32], info: &[u8]) -> (Zeroizing<[u8; 32]>, Zeroizing<[u8; 32]>) {
    let mut okm = Zeroizing::new([0u8; 64]);
    // 64 B je hluboko pod limitem HKDF-SHA256 (8160 B), expand tu nemůže selhat
    if Hkdf::<Sha256>::new(Some(th), shared).expand(info, &mut *okm).is_err() {
        unreachable!("HKDF output length is within bounds");
    }
    let mut first = Zeroizing::new([0u8; 32]);
    let mut second = Zeroizing::new([0u8; 32]);
    first.copy_from_slice(&okm[..32]);
    second.copy_from_slice(&okm[32..]);
    (first, second)
}

// --- padding ---------------------------------------------------------------------------------------------------

/// Velikost po paddingu: `4 + len` zaokrouhleno nahoru na násobek 256, nejméně 256.
pub fn padded_len(len: usize) -> usize {
    (4 + len).div_ceil(PAD_BLOCK).max(1) * PAD_BLOCK
}

pub fn pad(data: &[u8]) -> Result<Vec<u8>, ChannelError> {
    let len = u32::try_from(data.len()).map_err(|_| ChannelError::TooLarge)?;
    let mut out = Vec::with_capacity(padded_len(data.len()));
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(data);
    out.resize(padded_len(data.len()), 0);
    Ok(out)
}

/// Odstraní padding. Přijímá jen kanonický tvar (správná délka, samé nuly), jinak by šlo skrýt data v paddingu.
pub fn unpad(mut padded: Vec<u8>) -> Result<Vec<u8>, ChannelError> {
    let header: [u8; 4] = padded.get(..4).and_then(|h| h.try_into().ok()).ok_or(ChannelError::Padding)?;
    let len = u32::from_be_bytes(header) as usize;
    if len > padded.len() - 4 || padded.len() != padded_len(len) || padded[4 + len..].iter().any(|b| *b != 0) {
        return Err(ChannelError::Padding);
    }
    padded.truncate(4 + len);
    padded.drain(..4);
    Ok(padded)
}

// --- klient <-> node (§14.1) -------------------------------------------------------------------------------------

/// Hash přepisu handshaku klient ↔ node. Váže výzvu, oba efemerní klíče, identitu nodu a jeho `node_id`.
pub fn client_transcript(challenge: &[u8; 32], ce: &[u8; 32], ne: &[u8; 32], node_pk: &[u8; 32], node_id: &str) -> [u8; 32] {
    sha256_parts(&[CHAN_LABEL, &[0], challenge, ce, ne, node_pk, node_id.as_bytes()])
}

/// Jednosměrný šifrovaný proud rámců (WebSocket je uspořádaný, takže stačí čítač).
pub struct FrameCipher {
    cipher: ChaCha20Poly1305,
    counter: u64,
    aad: Vec<u8>,
}

impl FrameCipher {
    fn new(key: &[u8; 32], th: &[u8; 32]) -> Self {
        let mut aad = CHAN_FRAME_AAD.to_vec();
        aad.extend_from_slice(th);
        Self { cipher: ChaCha20Poly1305::new(Key::from_slice(key)), counter: 0, aad }
    }

    fn nonce(&self) -> Result<Nonce, ChannelError> {
        if self.counter >= MAX_FRAMES {
            return Err(ChannelError::Exhausted);
        }
        let mut n = [0u8; 12];
        n[4..].copy_from_slice(&self.counter.to_be_bytes());
        Ok(*Nonce::from_slice(&n))
    }

    pub fn seal(&mut self, data: &[u8]) -> Result<Vec<u8>, ChannelError> {
        let nonce = self.nonce()?;
        let out = self
            .cipher
            .encrypt(&nonce, Payload { msg: &pad(data)?, aad: &self.aad })
            .map_err(|_| ChannelError::Decrypt)?;
        self.counter += 1;
        Ok(out)
    }

    pub fn open(&mut self, frame: &[u8]) -> Result<Vec<u8>, ChannelError> {
        let nonce = self.nonce()?;
        let padded = self.cipher.decrypt(&nonce, Payload { msg: frame, aad: &self.aad }).map_err(|_| ChannelError::Decrypt)?;
        let out = unpad(padded)?;
        self.counter += 1;
        Ok(out)
    }
}

/// Oba směry navázaného kanálu.
pub struct Channel {
    pub send: FrameCipher,
    pub recv: FrameCipher,
}

impl Channel {
    /// Strana klienta: posílá klíčem `c2n`, přijímá `n2c`.
    pub fn for_client(shared: &[u8; 32], th: &[u8; 32]) -> Self {
        let (c2n, n2c) = derive(shared, th, CHAN_KEYS_INFO);
        Self { send: FrameCipher::new(&c2n, th), recv: FrameCipher::new(&n2c, th) }
    }

    pub fn for_node(shared: &[u8; 32], th: &[u8; 32]) -> Self {
        let (c2n, n2c) = derive(shared, th, CHAN_KEYS_INFO);
        Self { send: FrameCipher::new(&n2c, th), recv: FrameCipher::new(&c2n, th) }
    }
}

// --- node <-> node (§14.2) ---------------------------------------------------------------------------------------

/// Hash přepisu handshaku node ↔ node. `request_nonce` je 16 B `n` z podepsané hlavičky `/n/hs`.
pub fn n2n_transcript(
    initiator_pk: &[u8; 32],
    responder_pk: &[u8; 32],
    request_nonce: &[u8; 16],
    ce: &[u8; 32],
    ne: &[u8; 32],
    sid: &[u8; 16],
) -> [u8; 32] {
    sha256_parts(&[N2N_LABEL, &[0], initiator_pk, responder_pk, request_nonce, ce, ne, sid])
}

/// Klíče session mezi nody: `(initiator → responder, responder → initiator)`.
pub struct SessionKeys {
    pub i2r: Zeroizing<[u8; 32]>,
    pub r2i: Zeroizing<[u8; 32]>,
}

impl SessionKeys {
    pub fn derive(shared: &[u8; 32], th: &[u8; 32]) -> Self {
        let (i2r, r2i) = derive(shared, th, N2N_KEYS_INFO);
        Self { i2r, r2i }
    }
}

fn n2n_aad(label: &[u8], sid: &[u8; 16], request_nonce: Option<&[u8; 24]>) -> Vec<u8> {
    let mut aad = label.to_vec();
    aad.extend_from_slice(sid);
    if let Some(n) = request_nonce {
        aad.extend_from_slice(n);
    }
    aad
}

fn x_seal(key: &[u8; 32], nonce: &[u8; 24], aad: &[u8], data: &[u8]) -> Result<Vec<u8>, ChannelError> {
    XChaCha20Poly1305::new(Key::from_slice(key))
        .encrypt(XNonce::from_slice(nonce), Payload { msg: &pad(data)?, aad })
        .map_err(|_| ChannelError::Decrypt)
}

fn x_open(key: &[u8; 32], nonce: &[u8; 24], aad: &[u8], ct: &[u8]) -> Result<Vec<u8>, ChannelError> {
    let padded = XChaCha20Poly1305::new(Key::from_slice(key))
        .decrypt(XNonce::from_slice(nonce), Payload { msg: ct, aad })
        .map_err(|_| ChannelError::Decrypt)?;
    unpad(padded)
}

pub fn seal_request(key: &[u8; 32], sid: &[u8; 16], nonce: &[u8; 24], data: &[u8]) -> Result<Vec<u8>, ChannelError> {
    x_seal(key, nonce, &n2n_aad(N2N_REQ_AAD, sid, None), data)
}

pub fn open_request(key: &[u8; 32], sid: &[u8; 16], nonce: &[u8; 24], ct: &[u8]) -> Result<Vec<u8>, ChannelError> {
    x_open(key, nonce, &n2n_aad(N2N_REQ_AAD, sid, None), ct)
}

/// Odpověď je vázaná na `nonce` požadavku, takže ji nelze přiřadit k jinému požadavku.
pub fn seal_response(key: &[u8; 32], sid: &[u8; 16], request_nonce: &[u8; 24], nonce: &[u8; 24], data: &[u8]) -> Result<Vec<u8>, ChannelError> {
    x_seal(key, nonce, &n2n_aad(N2N_RES_AAD, sid, Some(request_nonce)), data)
}

pub fn open_response(key: &[u8; 32], sid: &[u8; 16], request_nonce: &[u8; 24], nonce: &[u8; 24], ct: &[u8]) -> Result<Vec<u8>, ChannelError> {
    x_open(key, nonce, &n2n_aad(N2N_RES_AAD, sid, Some(request_nonce)), ct)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn padding_is_canonical() {
        for len in [0usize, 1, 251, 252, 253, 508, 1000] {
            let data = vec![7u8; len];
            let padded = pad(&data).unwrap();
            assert_eq!(padded.len() % PAD_BLOCK, 0);
            assert_eq!(unpad(padded).unwrap(), data);
        }
        let mut bad = pad(b"abc").unwrap();
        *bad.last_mut().unwrap() = 1;
        assert_eq!(unpad(bad), Err(ChannelError::Padding));
        assert_eq!(unpad(vec![0, 0, 1, 0]), Err(ChannelError::Padding));
        let mut long = pad(b"abc").unwrap();
        long.extend_from_slice(&[0u8; PAD_BLOCK]);
        assert_eq!(unpad(long), Err(ChannelError::Padding), "extra padding blocks are not canonical");
    }

    #[test]
    fn channel_roundtrip_detects_reorder_replay_and_tamper() {
        let (a, b) = (Ephemeral::generate(), Ephemeral::generate());
        let (pa, pb) = (a.public(), b.public());
        let th = [9u8; 32];
        let mut client = Channel::for_client(&a.diffie_hellman(&pb).unwrap(), &th);
        let mut node = Channel::for_node(&b.diffie_hellman(&pa).unwrap(), &th);
        let f0 = client.send.seal(b"one").unwrap();
        let f1 = client.send.seal(b"two").unwrap();
        assert_eq!(node.recv.open(&f1), Err(ChannelError::Decrypt), "out of order frame");
        assert_eq!(node.recv.open(&f0).unwrap(), b"one");
        assert_eq!(node.recv.open(&f0), Err(ChannelError::Decrypt), "replay");
        let mut tampered = f1.clone();
        tampered[3] ^= 1;
        assert_eq!(node.recv.open(&tampered), Err(ChannelError::Decrypt));
        assert_eq!(node.recv.open(&f1).unwrap(), b"two");
        let back = node.send.seal(b"reply").unwrap();
        assert_eq!(client.recv.open(&back).unwrap(), b"reply");
        assert_eq!(client.recv.open(&f0), Err(ChannelError::Decrypt), "frames are not interchangeable between directions");
    }

    #[test]
    fn different_transcript_yields_different_keys() {
        let (a, b) = (Ephemeral::generate(), Ephemeral::generate());
        let pb = b.public();
        let shared = a.diffie_hellman(&pb).unwrap();
        let mut c1 = Channel::for_client(&shared, &[1; 32]);
        let mut c2 = Channel::for_node(&shared, &[2; 32]);
        let f = c1.send.seal(b"x").unwrap();
        assert!(c2.recv.open(&f).is_err());
    }

    #[test]
    fn low_order_points_are_rejected() {
        assert_eq!(Ephemeral::generate().diffie_hellman(&[0u8; 32]).err(), Some(ChannelError::BadPublicKey));
    }

    #[test]
    fn frame_counter_exhaustion_is_fatal() {
        let mut c = Channel::for_client(&[3; 32], &[4; 32]);
        c.send.counter = MAX_FRAMES;
        assert_eq!(c.send.seal(b"x"), Err(ChannelError::Exhausted));
    }

    #[test]
    fn response_is_bound_to_request() {
        let (key, sid) = ([5u8; 32], [6u8; 16]);
        let (rq1, rq2, n) = ([1u8; 24], [2u8; 24], [3u8; 24]);
        let ct = seal_response(&key, &sid, &rq1, &n, b"{}").unwrap();
        assert!(open_response(&key, &sid, &rq1, &n, &ct).is_ok());
        assert!(open_response(&key, &sid, &rq2, &n, &ct).is_err());
        assert!(open_request(&key, &sid, &n, &ct).is_err(), "request and response contexts differ");
    }
}
