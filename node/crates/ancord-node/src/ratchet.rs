//! Mailbox a E2E Double Ratchet mezi klienty (`spec/PROTOCOL.md` §13.1 a §13.2). Node ho nikdy nepočítá, sdílí ho s ním jen klient.

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use hkdf::Hkdf;
use rand::rngs::OsRng;
use rand::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use x25519_dalek::{PublicKey, StaticSecret};

use crate::encoding::{b64u_decode, b64u_encode};

/// Kolik klíčů přeskočených zpráv se najednou uloží; větší mezera je desync.
pub const MAX_SKIP: u32 = 200;
const TAG_LEN: usize = 16;
/// `dh(32) ‖ pn ‖ n ‖ seq`, čísla big-endian u32.
const HEADER_LEN: usize = 32 + 3 * 4;
/// Zašifrovaná hlavička (spec §13.2): `H` a tag.
const ENC_HEADER_LEN: usize = HEADER_LEN + TAG_LEN;

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
    #[error("no sending chain yet: the responder must first receive a message of the initiator")]
    NoSendChain,
    #[error("peer ratchet key is of low order")]
    WeakKey,
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

fn root_key(shared_secret: &[u8; 32]) -> [u8; 32] {
    expand(shared_secret, Some(b"ancord/v1/ratchet"), b"root")
}

/// Kořenový krok: z kořenového klíče a výstupu DH vznikne nový kořenový klíč, klíč řetězu a klíč hlavičky
/// následujícího řetězu téhož směru.
fn kdf_rk(root_key: &[u8; 32], dh_out: &[u8; 32]) -> ([u8; 32], [u8; 32], [u8; 32]) {
    let okm: [u8; 96] = expand(dh_out, Some(root_key), b"ancord/v1/ratchet/root");
    let (mut rk, mut ck, mut nhk) = ([0u8; 32], [0u8; 32], [0u8; 32]);
    rk.copy_from_slice(&okm[..32]);
    ck.copy_from_slice(&okm[32..64]);
    nhk.copy_from_slice(&okm[64..]);
    (rk, ck, nhk)
}

/// Klíč hlavičky prvního řetězu směru `dir` (spec §13.2).
fn initial_header_key(shared_secret: &[u8; 32], dir: Direction) -> [u8; 32] {
    expand(shared_secret, Some(b"ancord/v1/ratchet"), format!("header/{}", dir.as_str()).as_bytes())
}

/// Nonce hlavičky: `sha256("ancord/v1/hdr" ‖ 0x00 ‖ id)[0..12]`. `id` je náhodné a pro každou zprávu nové.
fn header_nonce(id: &[u8; 16]) -> [u8; 12] {
    let h: [u8; 32] = Sha256::new().chain_update(b"ancord/v1/hdr\0").chain_update(id).finalize().into();
    let mut n = [0u8; 12];
    n.copy_from_slice(&h[..12]);
    n
}

fn header_aad(mailbox_id: &[u8; 32], id: &[u8; 16]) -> Vec<u8> {
    let mut v = b"ancord/v1/hdr\0".to_vec();
    v.extend_from_slice(mailbox_id);
    v.extend_from_slice(id);
    v
}

fn seal_header(hk: &[u8; 32], mailbox_id: &[u8; 32], id: &[u8; 16], header: &Header) -> Result<Vec<u8>, RatchetError> {
    let plain = header.encode();
    ChaCha20Poly1305::new(Key::from_slice(hk))
        .encrypt(Nonce::from_slice(&header_nonce(id)), Payload { msg: &plain, aad: &header_aad(mailbox_id, id) })
        .map_err(|_| RatchetError::Auth)
}

fn open_header(hk: &[u8; 32], mailbox_id: &[u8; 32], id: &[u8; 16], enc: &[u8]) -> Option<Header> {
    let plain = ChaCha20Poly1305::new(Key::from_slice(hk))
        .decrypt(Nonce::from_slice(&header_nonce(id)), Payload { msg: enc, aad: &header_aad(mailbox_id, id) })
        .ok()?;
    Header::parse(&plain)
}

/// První ratchet klíč responderu, když strany nemají prekeys a sdílejí jen tajemství (`spec` §13.2).
pub fn bootstrap_secret(shared_secret: &[u8; 32]) -> StaticSecret {
    StaticSecret::from(expand::<32>(shared_secret, Some(b"ancord/v1/ratchet"), b"initial-dh"))
}

fn dh(secret: &StaticSecret, public: &[u8; 32]) -> Result<[u8; 32], RatchetError> {
    let shared = secret.diffie_hellman(&PublicKey::from(*public));
    if !shared.was_contributory() {
        return Err(RatchetError::WeakKey);
    }
    Ok(shared.to_bytes())
}

struct MessageKey {
    key: [u8; 32],
    nonce: [u8; 12],
}

/// Klíč nalezené zprávy a její `seq` z hlavičky.
struct Opened {
    key: [u8; 32],
    nonce: [u8; 12],
    seq: u32,
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

fn aad(mailbox_id: &[u8; 32], id: &[u8; 16], header: &[u8]) -> Vec<u8> {
    let mut v = b"ancord/v1/msg\0".to_vec();
    v.extend_from_slice(mailbox_id);
    v.extend_from_slice(id);
    v.extend_from_slice(header);
    v
}

struct Header {
    dh: [u8; 32],
    pn: u32,
    n: u32,
    seq: u32,
}

impl Header {
    fn encode(&self) -> [u8; HEADER_LEN] {
        let mut out = [0u8; HEADER_LEN];
        out[..32].copy_from_slice(&self.dh);
        out[32..36].copy_from_slice(&self.pn.to_be_bytes());
        out[36..40].copy_from_slice(&self.n.to_be_bytes());
        out[40..].copy_from_slice(&self.seq.to_be_bytes());
        out
    }

    fn parse(bytes: &[u8]) -> Option<Self> {
        let u32_at = |i: usize| bytes.get(i..i + 4).and_then(|b| b.try_into().ok()).map(u32::from_be_bytes);
        Some(Self { dh: bytes.get(..32)?.try_into().ok()?, pn: u32_at(32)?, n: u32_at(36)?, seq: u32_at(40)? })
    }
}

/// Uložený klíč zprávy, která v řetězu s klíčem hlavičky `hk` ještě nedorazila.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Skipped {
    hk: [u8; 32],
    n: u32,
    key: [u8; 32],
    nonce: [u8; 12],
}

/// Stav jedné strany konverzace: odesílá do jednoho mailboxu a čte z druhého. Serializuje se, aby klient přežil restart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ratchet {
    send_mailbox: [u8; 32],
    recv_mailbox: [u8; 32],
    root_key: [u8; 32],
    /// Tajný klíč vlastního ratchet páru.
    dhs: [u8; 32],
    dhr: Option<[u8; 32]>,
    cks: Option<[u8; 32]>,
    ckr: Option<[u8; 32]>,
    /// Klíče hlaviček: odesílacího a přijímacího řetězu a těch, které přijdou po dalším DH kroku.
    hks: Option<[u8; 32]>,
    hkr: Option<[u8; 32]>,
    nhks: [u8; 32],
    nhkr: [u8; 32],
    /// Proběhl DH krok, ale v novém odesílacím řetězu ještě nic neodešlo: protistrana nový klíč nezná (§13.2, keepalive).
    #[serde(default)]
    unanswered: bool,
    ns: u32,
    nr: u32,
    pn: u32,
    seq: u32,
    skipped: Vec<Skipped>,
}

impl Ratchet {
    fn new(shared_secret: &[u8; 32], send: Direction, dhs: &StaticSecret) -> Self {
        Self {
            send_mailbox: mailbox_id(shared_secret, send),
            recv_mailbox: mailbox_id(shared_secret, send.opposite()),
            root_key: root_key(shared_secret),
            dhs: dhs.to_bytes(),
            dhr: None,
            cks: None,
            ckr: None,
            // první řetězy obou směrů mají klíč hlavičky ze sdíleného tajemství; responder ho pro svůj směr použije
            // až po DH kroku (`nhks`), iniciátor hned
            hks: None,
            hkr: None,
            nhks: initial_header_key(shared_secret, send),
            nhkr: initial_header_key(shared_secret, send.opposite()),
            unanswered: false,
            ns: 0,
            nr: 0,
            pn: 0,
            seq: 0,
            skipped: Vec::new(),
        }
    }

    /// Iniciátor: zná první ratchet klíč responderu (jeho prekey, resp. `bootstrap_secret`) a může psát hned.
    pub fn initiator(shared_secret: &[u8; 32], peer_ratchet_key: &[u8; 32]) -> Result<Self, RatchetError> {
        Self::initiator_with_rng(shared_secret, peer_ratchet_key, &mut OsRng)
    }

    pub fn initiator_with_rng(
        shared_secret: &[u8; 32],
        peer_ratchet_key: &[u8; 32],
        rng: &mut (impl RngCore + CryptoRng),
    ) -> Result<Self, RatchetError> {
        let dhs = StaticSecret::random_from_rng(rng);
        let mut state = Self::new(shared_secret, Direction::InitiatorToResponder, &dhs);
        let (rk, cks, nhks) = kdf_rk(&state.root_key, &dh(&dhs, peer_ratchet_key)?);
        state.root_key = rk;
        state.cks = Some(cks);
        state.hks = Some(state.nhks);
        state.nhks = nhks;
        state.dhr = Some(*peer_ratchet_key);
        Ok(state)
    }

    /// Responder: `ratchet_secret` je tajný klíč k prvnímu ratchet klíči, který iniciátor zná. Psát může až po první zprávě.
    pub fn responder(shared_secret: &[u8; 32], ratchet_secret: &StaticSecret) -> Self {
        Self::new(shared_secret, Direction::ResponderToInitiator, ratchet_secret)
    }

    /// Obě strany znají jen sdílené tajemství (bez prekeys): první ratchet klíč responderu je z něj odvozený.
    pub fn from_shared_secret(shared_secret: &[u8; 32], initiator: bool) -> Result<Self, RatchetError> {
        let boot = bootstrap_secret(shared_secret);
        if initiator {
            Self::initiator(shared_secret, &PublicKey::from(&boot).to_bytes())
        } else {
            Ok(Self::responder(shared_secret, &boot))
        }
    }

    /// Mailbox, do kterého tato strana píše.
    pub fn send_mailbox_id(&self) -> [u8; 32] {
        self.send_mailbox
    }

    /// Mailbox, ze kterého tato strana čte.
    pub fn recv_mailbox_id(&self) -> [u8; 32] {
        self.recv_mailbox
    }

    /// Zašifruje zprávu `id` a posune řetěz. Vrací `ct` (b64u) pro pole `ct` zprávy `send`.
    pub fn seal(&mut self, id: &[u8; 16], plaintext: &[u8]) -> Result<String, RatchetError> {
        let (Some(ck), Some(hk)) = (self.cks, self.hks) else { return Err(RatchetError::NoSendChain) };
        let next_ns = self.ns.checked_add(1).ok_or(RatchetError::Exhausted)?;
        let next_seq = self.seq.checked_add(1).ok_or(RatchetError::Exhausted)?;
        let header = Header { dh: PublicKey::from(&StaticSecret::from(self.dhs)).to_bytes(), pn: self.pn, n: self.ns, seq: self.seq };
        let enc_header = seal_header(&hk, &self.send_mailbox, id, &header)?;
        let mk = message_key(&ck);
        let body = ChaCha20Poly1305::new(Key::from_slice(&mk.key))
            .encrypt(Nonce::from_slice(&mk.nonce), Payload { msg: plaintext, aad: &aad(&self.send_mailbox, id, &enc_header) })
            .map_err(|_| RatchetError::Auth)?;
        self.cks = Some(next_chain_key(&ck));
        self.ns = next_ns;
        self.seq = next_seq;
        self.unanswered = false;
        let mut wire = enc_header;
        wire.extend_from_slice(&body);
        Ok(b64u_encode(&wire))
    }

    /// Proběhl DH krok, na který tato strana zatím nic neposlala. Dokud něco nepošle, protistrana její nový klíč
    /// nezná a konverzace se po úniku stavu neuzdraví; klient proto po čase pošle prázdnou zprávu (keepalive, §13.2).
    pub fn needs_keepalive(&self) -> bool {
        self.unanswered && self.cks.is_some()
    }

    /// Dešifruje zprávu. Stav (včetně DH kroku) se změní až po ověření tagu, takže podvržená zpráva nic neposune.
    pub fn open(&mut self, id: &[u8; 16], ct: &str) -> Result<Vec<u8>, RatchetError> {
        self.open_numbered(id, ct).map(|(_, plaintext)| plaintext)
    }

    /// Jako `open`, ale vrací i pořadové číslo zprávy `seq`. Podle něj klient řadí zprávy: `ts` má jen sekundovou
    /// přesnost a `n` se s každým DH krokem vrací na nulu.
    pub fn open_numbered(&mut self, id: &[u8; 16], ct: &str) -> Result<(u32, Vec<u8>), RatchetError> {
        self.open_numbered_with_rng(id, ct, &mut OsRng)
    }

    pub fn open_numbered_with_rng(
        &mut self,
        id: &[u8; 16],
        ct: &str,
        rng: &mut (impl RngCore + CryptoRng),
    ) -> Result<(u32, Vec<u8>), RatchetError> {
        let wire = b64u_decode(ct).ok_or(RatchetError::Malformed)?;
        if wire.len() < ENC_HEADER_LEN + TAG_LEN {
            return Err(RatchetError::Malformed);
        }
        let (head, body) = wire.split_at(ENC_HEADER_LEN);
        let aad = aad(&self.recv_mailbox, id, head);

        // pracuje se na kopii; přijme se, až když sedí tag
        let mut next = self.clone();
        let mk = next.message_key_for(id, head, rng)?;
        let plaintext = ChaCha20Poly1305::new(Key::from_slice(&mk.key))
            .decrypt(Nonce::from_slice(&mk.nonce), Payload { msg: body, aad: &aad })
            .map_err(|_| RatchetError::Auth)?;
        *self = next;
        Ok((mk.seq, plaintext))
    }

    /// Najde klíč zprávy podle zašifrované hlavičky: nejdřív klíče hlaviček uložených přeskočených zpráv, pak
    /// aktuální přijímací řetěz, pak klíč dalšího řetězu (ten znamená DH krok).
    fn message_key_for(&mut self, id: &[u8; 16], enc: &[u8], rng: &mut (impl RngCore + CryptoRng)) -> Result<Opened, RatchetError> {
        let mut tried: Vec<[u8; 32]> = Vec::new();
        let mut current = None;
        for hk in self.skipped.iter().map(|s| s.hk) {
            if tried.contains(&hk) {
                continue;
            }
            tried.push(hk);
            if let Some(header) = open_header(&hk, &self.recv_mailbox, id, enc) {
                if let Some(pos) = self.skipped.iter().position(|s| s.hk == hk && s.n == header.n) {
                    let s = self.skipped.remove(pos);
                    return Ok(Opened { key: s.key, nonce: s.nonce, seq: header.seq });
                }
                // přeskočené klíče aktuálního řetězu mají jeho klíč hlavičky: zpráva pokračuje v řetězu
                if Some(hk) != self.hkr {
                    return Err(RatchetError::Replayed);
                }
                current = Some(header);
                break;
            }
        }
        let current = current.or_else(|| self.hkr.and_then(|hk| open_header(&hk, &self.recv_mailbox, id, enc)));
        let header = match current {
            Some(h) => h,
            None => {
                let h = open_header(&self.nhkr, &self.recv_mailbox, id, enc).ok_or(RatchetError::Auth)?;
                self.skip_until(h.pn)?;
                self.dh_step(&h.dh, rng)?;
                h
            }
        };
        if header.n < self.nr {
            return Err(RatchetError::Replayed);
        }
        self.skip_until(header.n)?;
        let ck = self.ckr.ok_or(RatchetError::Malformed)?;
        self.nr = header.n.checked_add(1).ok_or(RatchetError::Exhausted)?;
        self.ckr = Some(next_chain_key(&ck));
        let mk = message_key(&ck);
        Ok(Opened { key: mk.key, nonce: mk.nonce, seq: header.seq })
    }

    /// Uloží klíče zpráv přijímacího řetězu do `until` (výjimečně), aby šly otevřít, až dorazí.
    fn skip_until(&mut self, until: u32) -> Result<(), RatchetError> {
        let (Some(mut ck), Some(hkr)) = (self.ckr, self.hkr) else { return Ok(()) };
        if until.saturating_sub(self.nr) > MAX_SKIP {
            return Err(RatchetError::Desync);
        }
        while self.nr < until {
            let mk = message_key(&ck);
            self.skipped.push(Skipped { hk: hkr, n: self.nr, key: mk.key, nonce: mk.nonce });
            ck = next_chain_key(&ck);
            self.nr += 1;
        }
        self.ckr = Some(ck);
        let excess = self.skipped.len().saturating_sub(MAX_SKIP as usize);
        self.skipped.drain(..excess);
        Ok(())
    }

    /// DH krok: nový přijímací řetěz podle klíče protistrany, pak nový vlastní pár a nový odesílací řetěz.
    fn dh_step(&mut self, peer: &[u8; 32], rng: &mut (impl RngCore + CryptoRng)) -> Result<(), RatchetError> {
        let (rk, ckr, nhkr) = kdf_rk(&self.root_key, &dh(&StaticSecret::from(self.dhs), peer)?);
        let fresh = StaticSecret::random_from_rng(rng);
        let (rk, cks, nhks) = kdf_rk(&rk, &dh(&fresh, peer)?);
        self.pn = self.ns;
        self.ns = 0;
        self.nr = 0;
        self.dhr = Some(*peer);
        self.root_key = rk;
        self.ckr = Some(ckr);
        self.cks = Some(cks);
        self.hks = Some(self.nhks);
        self.hkr = Some(self.nhkr);
        self.nhks = nhks;
        self.nhkr = nhkr;
        self.dhs = fresh.to_bytes();
        self.unanswered = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SS: [u8; 32] = [9; 32];

    fn pair() -> (Ratchet, Ratchet) {
        (Ratchet::from_shared_secret(&SS, true).unwrap(), Ratchet::from_shared_secret(&SS, false).unwrap())
    }

    fn id(i: u8) -> [u8; 16] {
        [i; 16]
    }

    #[test]
    fn roundtrip_both_directions_through_several_dh_steps() {
        let (mut a, mut b) = pair();
        for round in 0u8..4 {
            let ct = a.seal(&id(round), &[round, 1]).unwrap();
            assert_eq!(b.open(&id(round), &ct).unwrap(), vec![round, 1]);
            let ct = b.seal(&id(round + 100), &[round, 2]).unwrap();
            assert_eq!(a.open(&id(round + 100), &ct).unwrap(), vec![round, 2]);
        }
    }

    #[test]
    fn responder_cannot_send_before_the_first_message() {
        let (mut a, mut b) = pair();
        assert_eq!(b.seal(&id(1), b"x"), Err(RatchetError::NoSendChain));
        let ct = a.seal(&id(2), b"hi").unwrap();
        b.open(&id(2), &ct).unwrap();
        assert!(b.seal(&id(1), b"x").is_ok());
    }

    #[test]
    fn out_of_order_within_a_chain_and_replay() {
        let (mut a, mut b) = pair();
        let cts: Vec<_> = (0u8..4).map(|i| a.seal(&id(i), &[i]).unwrap()).collect();
        assert_eq!(b.open(&id(2), &cts[2]).unwrap(), vec![2]);
        assert_eq!(b.open(&id(0), &cts[0]).unwrap(), vec![0]);
        assert_eq!(b.open(&id(3), &cts[3]).unwrap(), vec![3]);
        assert_eq!(b.open(&id(1), &cts[1]).unwrap(), vec![1]);
        assert_eq!(b.open(&id(1), &cts[1]), Err(RatchetError::Replayed));
        assert_eq!(b.open(&id(3), &cts[3]), Err(RatchetError::Replayed));
    }

    #[test]
    fn message_of_the_previous_chain_arrives_after_a_dh_step() {
        let (mut a, mut b) = pair();
        let (m0, m1, m2) = (a.seal(&id(0), b"m0").unwrap(), a.seal(&id(1), b"m1").unwrap(), a.seal(&id(2), b"m2").unwrap());
        b.open(&id(0), &m0).unwrap();
        let r = b.seal(&id(10), b"r").unwrap();
        a.open(&id(10), &r).unwrap();
        // Alice udělala DH krok; m3 patří do nového řetězu, m1 a m2 ještě nedorazily
        let m3 = a.seal(&id(3), b"m3").unwrap();
        let (seq, plain) = b.open_numbered(&id(3), &m3).unwrap();
        assert_eq!((seq, plain.as_slice()), (3, b"m3".as_slice()));
        assert_eq!(b.open_numbered(&id(2), &m2).unwrap(), (2, b"m2".to_vec()));
        assert_eq!(b.open_numbered(&id(1), &m1).unwrap(), (1, b"m1".to_vec()));
        assert!(b.open(&id(1), &m1).is_err(), "a consumed key of an old chain cannot be reused");
    }

    #[test]
    fn forged_message_does_not_advance_state() {
        let (mut a, mut b) = pair();
        let ct = a.seal(&id(1), b"real").unwrap();
        // jiné id = jiné AAD; DH krok na kopii se zahodí
        assert_eq!(b.open(&id(2), &ct), Err(RatchetError::Auth));
        assert_eq!(b.open(&id(1), &ct).unwrap(), b"real");
        // totéž s poškozenou hlavičkou nové větve
        let mut wire = b64u_decode(&a.seal(&id(3), b"x").unwrap()).unwrap();
        wire[3] ^= 1;
        assert_eq!(b.open(&id(3), &b64u_encode(&wire)), Err(RatchetError::Auth));
        let ct = a.seal(&id(4), b"y").unwrap();
        assert_eq!(b.open(&id(4), &ct).unwrap(), b"y");
    }

    #[test]
    fn wrong_mailbox_or_secret_fails() {
        let (mut a, _) = pair();
        let ct = a.seal(&id(3), b"x").unwrap();
        let mut other_secret = Ratchet::from_shared_secret(&[1; 32], false).unwrap();
        assert_eq!(other_secret.open(&id(3), &ct), Err(RatchetError::Auth));
        // vlastní zprávu strana otevřít nemůže (čte z druhého mailboxu)
        let mut a2 = Ratchet::from_shared_secret(&SS, true).unwrap();
        assert_eq!(a2.open(&id(3), &ct), Err(RatchetError::Auth));
    }

    #[test]
    fn gap_limit() {
        let (mut a, mut b) = pair();
        let mut last = String::new();
        for i in 0..=(MAX_SKIP + 1) {
            last = a.seal(&id((i % 251) as u8), b"m").unwrap();
        }
        assert_eq!(b.open(&id(((MAX_SKIP + 1) % 251) as u8), &last), Err(RatchetError::Desync));
    }

    #[test]
    fn garbage_is_rejected() {
        let (_, mut b) = pair();
        assert_eq!(b.open(&[0; 16], "!!"), Err(RatchetError::Malformed));
        assert_eq!(b.open(&[0; 16], "AAAA"), Err(RatchetError::Malformed));
        // hlavička, kterou nejde otevřít žádným klíčem hlavičky
        let junk = b64u_encode(&[0u8; ENC_HEADER_LEN + TAG_LEN]);
        assert_eq!(b.open(&[0; 16], &junk), Err(RatchetError::Auth));
    }

    #[test]
    fn state_survives_serialization() {
        let (mut a, mut b) = pair();
        let ct = a.seal(&id(4), b"a").unwrap();
        b.open(&id(4), &ct).unwrap();
        let r = b.seal(&id(5), b"r").unwrap();
        let mut a2: Ratchet = serde_json::from_str(&serde_json::to_string(&a).unwrap()).unwrap();
        let mut b2: Ratchet = serde_json::from_str(&serde_json::to_string(&b).unwrap()).unwrap();
        assert_eq!(a2.open(&id(5), &r).unwrap(), b"r");
        let ct2 = a2.seal(&id(6), b"b").unwrap();
        assert_eq!(b2.open(&id(6), &ct2).unwrap(), b"b");
    }

    /// Anchor v hlavičce nevidí ratchet klíč ani čítače (spec §13.2).
    #[test]
    fn header_hides_ratchet_key_and_counters() {
        let (mut a, mut b) = pair();
        let dh_pub = PublicKey::from(&StaticSecret::from(a.dhs)).to_bytes();
        let c1 = b64u_decode(&a.seal(&id(1), b"x").unwrap()).unwrap();
        let c2 = b64u_decode(&a.seal(&id(2), b"x").unwrap()).unwrap();
        assert!(!c1.windows(32).any(|w| w == dh_pub));
        assert_ne!(c1[..ENC_HEADER_LEN], c2[..ENC_HEADER_LEN], "same chain, but headers must not be linkable");
        assert_eq!(c1.len(), ENC_HEADER_LEN + 1 + TAG_LEN);
        assert_eq!(b.open(&id(2), &b64u_encode(&c2)).unwrap(), b"x");
        assert_eq!(b.open(&id(1), &b64u_encode(&c1)).unwrap(), b"x");
    }

    #[test]
    fn keepalive_is_needed_after_a_dh_step_until_something_is_sent() {
        let (mut a, mut b) = pair();
        assert!(!a.needs_keepalive());
        let m = a.seal(&id(1), b"m").unwrap();
        b.open(&id(1), &m).unwrap();
        assert!(b.needs_keepalive(), "Bob has a fresh pair Alice does not know");
        // další zpráva téhož řetězu nic nemění
        let m2 = a.seal(&id(2), b"m").unwrap();
        b.open(&id(2), &m2).unwrap();
        assert!(b.needs_keepalive());
        let k = b.seal(&id(3), b"").unwrap();
        assert!(!b.needs_keepalive());
        assert_eq!(a.open(&id(3), &k).unwrap(), b"");
        assert!(a.needs_keepalive());
        // stav s příznakem přežije serializaci
        let a2: Ratchet = serde_json::from_str(&serde_json::to_string(&a).unwrap()).unwrap();
        assert!(a2.needs_keepalive());
    }

    /// Post-compromise security: útočník s kopií stavu čte zprávy, jen dokud obě strany neudělají DH krok s novým párem.
    #[test]
    fn conversation_heals_after_a_state_leak() {
        let (mut a, mut b) = pair();
        let m0 = a.seal(&id(0), b"m0").unwrap();
        b.open(&id(0), &m0).unwrap();
        let r0 = b.seal(&id(1), b"r0").unwrap();
        a.open(&id(1), &r0).unwrap();

        // únik: útočník má stav obou stran a pak jen pasivně vidí šifrové texty
        let mut eve_a = a.clone();
        let mut eve_b = b.clone();

        // okno kompromitace: nové klíče ještě nikdo nevygeneroval
        let m1 = a.seal(&id(2), b"m1").unwrap();
        assert_eq!(eve_b.open(&id(2), &m1).unwrap(), b"m1");
        b.open(&id(2), &m1).unwrap();
        let r1 = b.seal(&id(3), b"r1").unwrap();
        assert_eq!(eve_a.open(&id(3), &r1).unwrap(), b"r1");
        a.open(&id(3), &r1).unwrap();

        // teď má Alice nový pár (A2) a Bob nový pár (B2), který útočník nezná: konverzace je uzdravená
        let m2 = a.seal(&id(4), b"m2").unwrap();
        assert!(eve_b.open(&id(4), &m2).is_err(), "Alice's new chain must be closed to the attacker");
        assert!(eve_b.clone().open(&id(4), &m2).is_err());
        assert_eq!(b.open(&id(4), &m2).unwrap(), b"m2");
        let r2 = b.seal(&id(5), b"r2").unwrap();
        assert!(eve_a.open(&id(5), &r2).is_err(), "Bob's new chain must be closed to the attacker");
        assert_eq!(a.open(&id(5), &r2).unwrap(), b"r2");
    }
}
