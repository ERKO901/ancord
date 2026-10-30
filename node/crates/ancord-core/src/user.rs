//! Klíče uživatele a dohoda sdíleného tajemství dvojice (`spec/PROTOCOL.md` §15.1 až §15.4). Používá klient.
//!
//! Dohoda je X3DH: iniciátor spojí tři až čtyři Diffieho-Hellmanovy výměny mezi dlouhodobými klíči, krátkodobým
//! podepsaným prekey, jednorázovým prekey a svým efemerním klíčem. Tajné klíče prekeys příjemce po použití, resp.
//! rotaci smaže, takže pozdější únik dlouhodobých klíčů neprozradí zprávy, jejichž prekeys už neexistují.

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use ed25519_dalek::SigningKey;
use hkdf::Hkdf;
use rand::rngs::OsRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use x25519_dalek::{PublicKey, StaticSecret};

use crate::encoding::{b64u_decode_array, b64u_encode};
use crate::profile::{Profile, MAX_OTKS};
use crate::wake::push_id_of;

/// Profil starší než tohle klient pro novou konverzaci nepřijme (anchor by mohl podstrčit zastaralý prekey).
pub const PROFILE_MAX_AGE_S: u64 = 14 * 86_400;
/// Jak často se rotuje podepsaný prekey.
pub const SPK_ROTATE_S: u64 = 7 * 86_400;
/// Jak dlouho se uchovává tajný klíč starého prekey (profil smí být `PROFILE_MAX_AGE_S` starý + rotace).
pub const SPK_KEEP_S: u64 = PROFILE_MAX_AGE_S + SPK_ROTATE_S;

const ENVELOPE_VERSION: u8 = 1;
const ENVELOPE_HEAD: usize = 1 + 8 + 32;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum UserError {
    #[error("peer profile has an invalid signature")]
    BadProfile,
    #[error("peer profile is too old; ask the peer to publish again")]
    StaleProfile,
    #[error("peer key is of low order")]
    WeakKey,
    #[error("cannot talk to yourself")]
    SelfConversation,
    #[error("invalid or foreign handshake envelope")]
    BadEnvelope,
    #[error("the prekey used by the handshake no longer exists")]
    PrekeyGone,
}

struct Spk {
    created: u64,
    secret: StaticSecret,
}

pub struct UserKeys {
    uk: SigningKey,
    ik: StaticSecret,
    /// Podepsané prekeys, nejnovější první.
    spks: Vec<Spk>,
    otks: Vec<StaticSecret>,
    /// Push klíč schránky (§15.11); jeho `push_id` je v profilu.
    push: SigningKey,
}

/// První ratchet klíč responderu (`spec` §13.2): iniciátor zná jeho veřejnou část, responder má tajnou.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FirstRatchetKey {
    /// Podepsaný prekey příjemce z jeho profilu (veřejný).
    Peer([u8; 32]),
    /// Tajný klíč vlastního podepsaného prekey, který obálka použila.
    Own([u8; 32]),
}

/// Sdílené tajemství dvojice, role a první ratchet klíč, kterou tato strana v konverzaci hraje.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pairwise {
    pub secret: [u8; 32],
    pub initiator: bool,
    pub ratchet_key: FirstRatchetKey,
}

/// Výsledek zahájení konverzace: tajemství pro iniciátora a obálka, kterou dostane příjemce do své schránky.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Initiation {
    pub pairwise: Pairwise,
    pub envelope: Vec<u8>,
}

/// Otevřená obálka: kdo se ozval a jaké prekeys použil. Z ní a z ověřeného profilu odesílatele vznikne tajemství.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Opened {
    pub peer_pk: [u8; 32],
    ek: [u8; 32],
    spk_idx: usize,
    otk: Option<[u8; 32]>,
}

#[derive(Serialize, Deserialize)]
struct SpkFile {
    created: u64,
    key: String,
}

#[derive(Serialize, Deserialize)]
struct StateFile {
    uk: String,
    ik: String,
    spks: Vec<SpkFile>,
    otks: Vec<String>,
    push: String,
}

/// Schránka, do které se píší úvodní obálky: `sha256("ancord/v1/inbox" ‖ 0x00 ‖ pk)`.
pub fn inbox_id(pk: &[u8; 32]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"ancord/v1/inbox");
    h.update([0]);
    h.update(pk);
    h.finalize().into()
}

/// Push klíč mailboxu konverzace (§15.11): `HKDF(salt = "ancord/v1/push", ikm = ss, info = mailbox_id)` jako Ed25519
/// seed. Příjemce pod jeho `push_id` registruje token, odesílatel `push_id` přikládá ke zprávám do tohoto mailboxu.
pub fn mailbox_push_key(shared_secret: &[u8; 32], mailbox_id: &[u8; 32]) -> SigningKey {
    SigningKey::from_bytes(&expand::<32>(b"ancord/v1/push", shared_secret, mailbox_id))
}

/// Adresa ke sdílení (QR kód, kopírování): `ancord:<pk>` (§15.10).
pub fn address_uri(pk: &[u8; 32]) -> String {
    format!("ancord:{}", b64u_encode(pk))
}

/// Adresa z textu `ancord:<pk>` nebo holého `pk`. Odmítne cokoli, co není platný veřejný klíč Ed25519.
pub fn parse_address(text: &str) -> Option<[u8; 32]> {
    let text = text.trim();
    let pk: [u8; 32] = b64u_decode_array(text.strip_prefix("ancord:").unwrap_or(text))?;
    ed25519_dalek::VerifyingKey::from_bytes(&pk).ok().map(|_| pk)
}

/// Bezpečnostní číslo dvojice (60 číslic): obě strany musí vidět totéž, jinak jim někdo podstrčil cizí klíč.
/// Porovnává se mimo síť (osobně, telefonem).
pub fn safety_number(a: &[u8; 32], b: &[u8; 32]) -> String {
    let (lo, hi) = if a < b { (a, b) } else { (b, a) };
    let mut h = Sha256::new();
    h.update(b"ancord/v1/safety");
    h.update([0]);
    h.update(lo);
    h.update(hi);
    let digest: [u8; 32] = h.finalize().into();
    let groups: Vec<String> = digest.as_chunks::<2>().0.iter().take(12).map(|c| format!("{:05}", u32::from(u16::from_be_bytes(*c)) % 100_000)).collect();
    groups.join(" ")
}

fn spk_id(spk: &[u8; 32]) -> [u8; 8] {
    let mut id = [0u8; 8];
    id.copy_from_slice(&Sha256::digest(spk)[..8]);
    id
}

fn expand<const N: usize>(salt: &[u8], ikm: &[u8], info: &[u8]) -> [u8; N] {
    let mut out = [0u8; N];
    // `expand` selže jen při délce > 255 · HashLen; N je malá konstanta
    let _ = Hkdf::<Sha256>::new(Some(salt), ikm).expand(info, &mut out);
    out
}

fn dh(secret: &StaticSecret, public: &[u8; 32]) -> Result<[u8; 32], UserError> {
    let shared = secret.diffie_hellman(&PublicKey::from(*public));
    if !shared.was_contributory() {
        return Err(UserError::WeakKey);
    }
    Ok(shared.to_bytes())
}

/// Spojí DH výstupy do tajemství. `0xFF…` na začátku odděluje výstup od jiných použití HKDF (jako v X3DH).
fn x3dh(dhs: &[[u8; 32]], pk_i: &[u8; 32], pk_r: &[u8; 32], ek: &[u8; 32], spk: &[u8; 32], otk: Option<&[u8; 32]>) -> [u8; 32] {
    let mut ikm = vec![0xFF; 32];
    for d in dhs {
        ikm.extend_from_slice(d);
    }
    let mut info = [pk_i.as_slice(), pk_r, ek, spk].concat();
    if let Some(otk) = otk {
        info.extend_from_slice(otk);
    }
    expand(b"ancord/v1/x3dh", &ikm, &info)
}

fn envelope_key(shared: &[u8; 32], inbox: &[u8; 32], ek: &[u8; 32], id: &[u8; 8]) -> ([u8; 32], [u8; 12], Vec<u8>) {
    let okm = expand::<44>(b"ancord/v1/envelope", shared, &[inbox.as_slice(), ek, id].concat());
    let aad = [b"ancord/v1/envelope".as_slice(), inbox, ek, id].concat();
    let (mut key, mut nonce) = ([0u8; 32], [0u8; 12]);
    key.copy_from_slice(&okm[..32]);
    nonce.copy_from_slice(&okm[32..]);
    (key, nonce, aad)
}

impl UserKeys {
    /// Nové klíče s prvním prekey a plnou zásobou jednorázových prekeys.
    pub fn generate(now: u64) -> Self {
        let mut keys = Self {
            uk: SigningKey::generate(&mut OsRng),
            ik: StaticSecret::random_from_rng(OsRng),
            spks: Vec::new(),
            otks: Vec::new(),
            push: SigningKey::generate(&mut OsRng),
        };
        keys.maintain(now);
        keys
    }

    /// Rotuje prekey, zahodí staré a doplní jednorázové. `true`, když se něco změnilo a je třeba uložit klíče a znovu
    /// zveřejnit profil.
    pub fn maintain(&mut self, now: u64) -> bool {
        let mut changed = false;
        if self.spks.first().is_none_or(|s| now.saturating_sub(s.created) >= SPK_ROTATE_S) {
            self.spks.insert(0, Spk { created: now, secret: StaticSecret::random_from_rng(OsRng) });
            changed = true;
        }
        let before = self.spks.len();
        let mut first = true;
        self.spks.retain(|s| std::mem::take(&mut first) || now.saturating_sub(s.created) < SPK_KEEP_S);
        changed |= self.spks.len() != before;
        while self.otks.len() < MAX_OTKS {
            self.otks.push(StaticSecret::random_from_rng(OsRng));
            changed = true;
        }
        changed
    }

    /// Klíče ze zadaných zárodků (testovací vektory): jeden prekey vytvořený v `created` a dané jednorázové prekeys.
    pub fn from_parts(uk: [u8; 32], ik: [u8; 32], spk: [u8; 32], created: u64, otks: &[[u8; 32]], push: [u8; 32]) -> Self {
        Self {
            uk: SigningKey::from_bytes(&uk),
            ik: StaticSecret::from(ik),
            spks: vec![Spk { created, secret: StaticSecret::from(spk) }],
            otks: otks.iter().map(|k| StaticSecret::from(*k)).collect(),
            push: SigningKey::from_bytes(&push),
        }
    }

    /// Obsah souboru s klíči (pro uložení s právy 0600).
    pub fn to_json(&self) -> String {
        let state = StateFile {
            uk: b64u_encode(&self.uk.to_bytes()),
            ik: b64u_encode(&self.ik.to_bytes()),
            spks: self.spks.iter().map(|s| SpkFile { created: s.created, key: b64u_encode(&s.secret.to_bytes()) }).collect(),
            otks: self.otks.iter().map(|k| b64u_encode(&k.to_bytes())).collect(),
            push: b64u_encode(&self.push.to_bytes()),
        };
        serde_json::to_string_pretty(&state).unwrap_or_default()
    }

    pub fn from_json(text: &str) -> Option<Self> {
        let f: StateFile = serde_json::from_str(text).ok()?;
        Some(Self {
            uk: SigningKey::from_bytes(&b64u_decode_array(&f.uk)?),
            ik: StaticSecret::from(b64u_decode_array::<32>(&f.ik)?),
            spks: f
                .spks
                .iter()
                .map(|s| Some(Spk { created: s.created, secret: StaticSecret::from(b64u_decode_array::<32>(&s.key)?) }))
                .collect::<Option<_>>()?,
            otks: f.otks.iter().map(|k| Some(StaticSecret::from(b64u_decode_array::<32>(k)?))).collect::<Option<_>>()?,
            push: SigningKey::from_bytes(&b64u_decode_array(&f.push)?),
        })
    }

    /// Adresa uživatele.
    pub fn pk(&self) -> [u8; 32] {
        self.uk.verifying_key().to_bytes()
    }

    /// Stráž vlastní schránky s obtížností razítka `bits` (§15.13).
    pub fn inbox_guard(&self, bits: u8, ts: u64) -> crate::guard::Guard {
        crate::guard::Guard::create(&self.uk, bits, ts)
    }

    pub fn bk_public(&self) -> [u8; 32] {
        PublicKey::from(&self.ik).to_bytes()
    }

    /// Push klíč schránky (pro registraci tokenu u Workeru).
    pub fn push_key(&self) -> &SigningKey {
        &self.push
    }

    /// `push_id` schránky, který je v profilu.
    pub fn push_id(&self) -> String {
        push_id_of(&self.push.verifying_key().to_bytes())
    }

    pub fn otk_count(&self) -> usize {
        self.otks.len()
    }

    /// Podepsaný profil s aktuálním prekey a zásobou jednorázových prekeys, s hashcash o `bits` bitech.
    pub fn profile(&self, ts: u64, bits: u32) -> Profile {
        let spk = self.spks.first().map(|s| PublicKey::from(&s.secret).to_bytes()).unwrap_or_default();
        let otks: Vec<[u8; 32]> = self.otks.iter().map(|k| PublicKey::from(k).to_bytes()).collect();
        Profile::create(&self.uk, &self.bk_public(), &spk, &otks, &self.push_id(), ts, bits)
    }

    /// Zahájí konverzaci s uživatelem, jehož profil klient získal z anchorů. Vrací tajemství a obálku pro jeho schránku.
    pub fn initiate(&self, peer: &Profile, now: u64) -> Result<Initiation, UserError> {
        let ek = StaticSecret::random_from_rng(OsRng);
        let pick = if peer.otks.is_empty() { None } else { Some(OsRng.gen_range(0..peer.otks.len())) };
        self.initiate_with(peer, now, &ek, pick)
    }

    /// Jako `initiate` s danými náhodnými volbami (testovací vektory).
    pub fn initiate_with(&self, peer: &Profile, now: u64, ek: &StaticSecret, otk_pick: Option<usize>) -> Result<Initiation, UserError> {
        if !peer.signature_valid() {
            return Err(UserError::BadProfile);
        }
        if peer.pk == self.pk() {
            return Err(UserError::SelfConversation);
        }
        if now.saturating_sub(peer.ts) > PROFILE_MAX_AGE_S || peer.ts.saturating_sub(now) > 86_400 {
            return Err(UserError::StaleProfile);
        }
        let otk = otk_pick.and_then(|i| peer.otks.get(i)).copied();
        let ek_pub = PublicKey::from(ek).to_bytes();
        let mut dhs = vec![dh(&self.ik, &peer.spk)?, dh(ek, &peer.bk)?, dh(ek, &peer.spk)?];
        if let Some(otk) = &otk {
            dhs.push(dh(ek, otk)?);
        }
        let secret = x3dh(&dhs, &self.pk(), &peer.pk, &ek_pub, &peer.spk, otk.as_ref());

        // obálka: kdo jsem a který jednorázový prekey jsem použil, zašifrováno klíčem, který zná jen příjemce
        let (id, inbox) = (spk_id(&peer.spk), inbox_id(&peer.pk));
        let (key, nonce, aad) = envelope_key(&dh(ek, &peer.spk)?, &inbox, &ek_pub, &id);
        let plain = [self.pk().as_slice(), &otk.unwrap_or([0; 32])].concat();
        let sealed = ChaCha20Poly1305::new(Key::from_slice(&key))
            .encrypt(Nonce::from_slice(&nonce), Payload { msg: &plain, aad: &aad })
            .map_err(|_| UserError::BadEnvelope)?;
        let mut envelope = vec![ENVELOPE_VERSION];
        envelope.extend_from_slice(&id);
        envelope.extend_from_slice(&ek_pub);
        envelope.extend_from_slice(&sealed);
        Ok(Initiation { pairwise: Pairwise { secret, initiator: true, ratchet_key: FirstRatchetKey::Peer(peer.spk) }, envelope })
    }

    /// Otevře obálku ze své schránky. Cizí, poškozenou nebo zastaralou obálku odmítne.
    pub fn open_envelope(&self, envelope: &[u8]) -> Result<Opened, UserError> {
        if envelope.len() <= ENVELOPE_HEAD || envelope[0] != ENVELOPE_VERSION {
            return Err(UserError::BadEnvelope);
        }
        let id: [u8; 8] = envelope[1..9].try_into().map_err(|_| UserError::BadEnvelope)?;
        let ek: [u8; 32] = envelope[9..41].try_into().map_err(|_| UserError::BadEnvelope)?;
        let spk_idx = self.spks.iter().position(|s| spk_id(&PublicKey::from(&s.secret).to_bytes()) == id).ok_or(UserError::PrekeyGone)?;
        let (key, nonce, aad) = envelope_key(&dh(&self.spks[spk_idx].secret, &ek)?, &inbox_id(&self.pk()), &ek, &id);
        let plain = ChaCha20Poly1305::new(Key::from_slice(&key))
            .decrypt(Nonce::from_slice(&nonce), Payload { msg: &envelope[ENVELOPE_HEAD..], aad: &aad })
            .map_err(|_| UserError::BadEnvelope)?;
        let (peer_pk, otk): ([u8; 32], [u8; 32]) = match (plain.get(..32), plain.get(32..)) {
            (Some(a), Some(b)) if plain.len() == 64 => (a.try_into().map_err(|_| UserError::BadEnvelope)?, b.try_into().map_err(|_| UserError::BadEnvelope)?),
            _ => return Err(UserError::BadEnvelope),
        };
        Ok(Opened { peer_pk, ek, spk_idx, otk: (otk != [0; 32]).then_some(otk) })
    }

    /// Dokončí dohodu na straně příjemce. `peer` je profil odesílatele získaný z anchorů; jednorázový prekey, který
    /// obálka použila, se tím nenávratně smaže (po uložení klíčů jeho hodnota neexistuje nikde).
    pub fn accept(&mut self, opened: &Opened, peer: &Profile) -> Result<Pairwise, UserError> {
        if !peer.signature_valid() || peer.pk != opened.peer_pk {
            return Err(UserError::BadProfile);
        }
        let spk_secret = &self.spks.get(opened.spk_idx).ok_or(UserError::PrekeyGone)?.secret;
        let spk_pub = PublicKey::from(spk_secret).to_bytes();
        let otk_pos = match &opened.otk {
            Some(pubkey) => Some(self.otks.iter().position(|k| &PublicKey::from(k).to_bytes() == pubkey).ok_or(UserError::PrekeyGone)?),
            None => None,
        };
        let mut dhs = vec![dh(spk_secret, &peer.bk)?, dh(&self.ik, &opened.ek)?, dh(spk_secret, &opened.ek)?];
        if let Some(i) = otk_pos {
            dhs.push(dh(&self.otks[i], &opened.ek)?);
        }
        let secret = x3dh(&dhs, &peer.pk, &self.pk(), &opened.ek, &spk_pub, opened.otk.as_ref());
        if let Some(i) = otk_pos {
            self.otks.remove(i);
        }
        Ok(Pairwise { secret, initiator: false, ratchet_key: FirstRatchetKey::Own(spk_secret.to_bytes()) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: u64 = 1_700_000_000;

    fn pair() -> (UserKeys, UserKeys) {
        (UserKeys::generate(NOW), UserKeys::generate(NOW))
    }

    #[test]
    fn both_sides_agree_and_the_one_time_prekey_is_consumed() {
        let (a, mut b) = pair();
        let pb = b.profile(NOW, 0);
        let init = a.initiate(&pb, NOW).unwrap();
        let opened = b.open_envelope(&init.envelope).unwrap();
        assert_eq!(opened.peer_pk, a.pk());
        let before = b.otk_count();
        let got = b.accept(&opened, &a.profile(NOW, 0)).unwrap();
        assert_eq!(got.secret, init.pairwise.secret);
        assert!(init.pairwise.initiator && !got.initiator);
        assert_eq!(b.otk_count(), before - 1, "used one-time prekey must be deleted");
        // stejná obálka podruhé: jednorázový prekey už neexistuje, nic se neobnoví
        assert_eq!(b.accept(&opened, &a.profile(NOW, 0)).unwrap_err(), UserError::PrekeyGone);
    }

    #[test]
    fn works_without_one_time_prekeys() {
        let (a, mut b) = pair();
        let mut resigned = UserKeys::from_json(&b.to_json()).unwrap();
        resigned.otks.clear();
        let pb = resigned.profile(NOW, 0);
        let init = a.initiate(&pb, NOW).unwrap();
        let opened = b.open_envelope(&init.envelope).unwrap();
        assert_eq!(b.accept(&opened, &a.profile(NOW, 0)).unwrap().secret, init.pairwise.secret);
    }

    #[test]
    fn each_conversation_gets_a_fresh_secret_and_third_parties_cannot_open() {
        let (a, b) = pair();
        let c = UserKeys::generate(NOW);
        let pb = b.profile(NOW, 0);
        let (i1, i2) = (a.initiate(&pb, NOW).unwrap(), a.initiate(&pb, NOW).unwrap());
        assert_ne!(i1.pairwise.secret, i2.pairwise.secret, "ephemeral key makes every handshake unique");
        assert_eq!(c.open_envelope(&i1.envelope).unwrap_err(), UserError::PrekeyGone);
        assert!(b.open_envelope(&i1.envelope).is_ok());
    }

    #[test]
    fn tampered_forged_and_foreign_input_is_refused() {
        let (a, mut b) = pair();
        let mallory = UserKeys::generate(NOW);
        let pb = b.profile(NOW, 0);
        let init = a.initiate(&pb, NOW).unwrap();
        let mut bad = init.envelope.clone();
        *bad.last_mut().unwrap() ^= 1;
        assert_eq!(b.open_envelope(&bad).unwrap_err(), UserError::BadEnvelope);
        assert_eq!(b.open_envelope(&init.envelope[..20]).unwrap_err(), UserError::BadEnvelope);
        // profil, který nepatří odesílateli obálky
        let opened = b.open_envelope(&init.envelope).unwrap();
        assert_eq!(b.accept(&opened, &mallory.profile(NOW, 0)).unwrap_err(), UserError::BadProfile);
        // podvržený klíč uvnitř profilu
        let mut forged = pb.clone();
        forged.spk = mallory.profile(NOW, 0).spk;
        assert_eq!(a.initiate(&forged, NOW).unwrap_err(), UserError::BadProfile);
        assert_eq!(a.initiate(&a.profile(NOW, 0), NOW).unwrap_err(), UserError::SelfConversation);
    }

    #[test]
    fn stale_profiles_are_refused() {
        let (a, b) = pair();
        let old = b.profile(NOW, 0);
        assert_eq!(a.initiate(&old, NOW + PROFILE_MAX_AGE_S + 1).unwrap_err(), UserError::StaleProfile);
        assert!(a.initiate(&old, NOW + PROFILE_MAX_AGE_S).is_ok());
    }

    #[test]
    fn rotation_deletes_old_prekeys_so_old_handshakes_cannot_be_opened_later() {
        let (a, mut b) = pair();
        let init = a.initiate(&b.profile(NOW, 0), NOW).unwrap();
        assert!(!b.maintain(NOW + 10), "nothing to do right after creation");
        assert!(b.maintain(NOW + SPK_ROTATE_S));
        assert!(b.open_envelope(&init.envelope).is_ok(), "the previous prekey is kept for a grace period");
        assert!(b.maintain(NOW + SPK_KEEP_S + SPK_ROTATE_S));
        assert_eq!(b.open_envelope(&init.envelope).unwrap_err(), UserError::PrekeyGone, "forward secrecy: the secret is gone");
    }

    #[test]
    fn keys_survive_a_roundtrip_through_the_file() {
        let (_, b) = pair();
        let again = UserKeys::from_json(&b.to_json()).unwrap();
        assert_eq!(again.pk(), b.pk());
        assert_eq!(again.profile(NOW, 0).spk, b.profile(NOW, 0).spk);
        assert_eq!(again.otk_count(), b.otk_count());
    }

    #[test]
    fn addresses_parse_with_or_without_the_scheme_and_reject_garbage() {
        let a = UserKeys::generate(NOW);
        let uri = address_uri(&a.pk());
        assert_eq!(parse_address(&uri), Some(a.pk()));
        assert_eq!(parse_address(&format!("  {}\n", b64u_encode(&a.pk()))), Some(a.pk()));
        assert_eq!(parse_address("ancord:"), None);
        assert_eq!(parse_address(&uri[..uri.len() - 1]), None);
        // 32 bajtů, ale ne bod křivky
        let not_a_point = (0u8..=255).map(|b| [b; 32]).find(|k| ed25519_dalek::VerifyingKey::from_bytes(k).is_err()).unwrap();
        assert_eq!(parse_address(&b64u_encode(&not_a_point)), None);
    }

    #[test]
    fn push_ids_are_stable_and_bound_to_the_mailbox() {
        let a = UserKeys::generate(NOW);
        let again = UserKeys::from_json(&a.to_json()).unwrap();
        assert_eq!(again.push_id(), a.push_id());
        assert_eq!(a.profile(NOW, 0).push_id, a.push_id());
        let (m1, m2) = ([1u8; 32], [2u8; 32]);
        let k = mailbox_push_key(&[7; 32], &m1);
        assert_eq!(k.to_bytes(), mailbox_push_key(&[7; 32], &m1).to_bytes());
        assert_ne!(k.to_bytes(), mailbox_push_key(&[7; 32], &m2).to_bytes());
        assert_ne!(k.to_bytes(), mailbox_push_key(&[8; 32], &m1).to_bytes());
    }

    #[test]
    fn safety_number_is_symmetric_and_pair_specific() {
        let (a, b) = pair();
        let c = UserKeys::generate(NOW);
        assert_eq!(safety_number(&a.pk(), &b.pk()), safety_number(&b.pk(), &a.pk()));
        assert_ne!(safety_number(&a.pk(), &b.pk()), safety_number(&a.pk(), &c.pk()));
        assert_eq!(safety_number(&a.pk(), &b.pk()).len(), 12 * 5 + 11);
    }

    #[test]
    fn profile_checks_time_and_proof_of_work() {
        let a = UserKeys::generate(NOW);
        let p = a.profile(1_000_000, 8);
        assert!(p.check(1_000_000, 8).is_ok());
        assert!(p.check(1_000_000 + 86_401, 8).is_err());
        let mut weak = p.clone();
        weak.pow = weak.pow.wrapping_add(1);
        assert!(weak.check(1_000_000, 20).is_err());
        let mut tampered = p.clone();
        tampered.otks.pop();
        assert!(tampered.check(1_000_000, 8).is_err());
        // push_id je pod podpisem: podvržený by přesměroval wake jinam
        let mut redirected = p;
        redirected.push_id = UserKeys::generate(NOW).push_id();
        assert!(redirected.check(1_000_000, 8).is_err());
    }
}
