//! Úložiště mailboxů na anchoru (redb, `spec/PROTOCOL.md` §13.5). Ukládá jen ciphertext a náhrobky doručených zpráv.

use std::fmt::Display;
use std::path::Path;

use redb::{Database, ReadableTable, ReadableTableMetadata, TableDefinition};

use crate::wake::PUSH_ID_LEN;

/// Klíč `mailbox_id(32) ‖ id(16)`, hodnota `state(1) ‖ ts_be_u64 ‖ exp_be_u64 ‖ ct`.
const MESSAGES: TableDefinition<&[u8], &[u8]> = TableDefinition::new("messages");
/// Počet nedoručených zpráv v mailboxu (pro strop), klíč `mailbox_id`.
const PENDING: TableDefinition<&[u8], u64> = TableDefinition::new("pending");

/// Veřejné profily uživatelů, klíč `pk(32)`, hodnota `ts ‖ exp ‖ pow` (každé u64 BE) `‖ bk(32) ‖ spk(32) ‖ sig(64) ‖ n(1) ‖ n × otk(32)`.
const PROFILES: TableDefinition<&[u8], &[u8]> = TableDefinition::new("profiles");
const PROFILE_FIXED_LEN: usize = 24 + 32 + 32 + 64 + PUSH_ID_LEN + 1;

/// Stráže schránek (`spec/PROTOCOL.md` §15.13), klíč `inbox_id(32)`, hodnota `pk(32) ‖ bits(1) ‖ ts ‖ exp` (u64 BE) `‖ sig(64)`.
const GUARDS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("guards");
const GUARD_LEN: usize = 32 + 1 + 16 + 64;

const KEY_LEN: usize = 48;
const HEADER_LEN: usize = 17;
const STATE_PENDING: u8 = 0;
const STATE_DELIVERED: u8 = 1;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum StoreError {
    #[error("mailbox is full")]
    Full,
    #[error("storage: {0}")]
    Io(String),
}

fn io<E: Display>(e: E) -> StoreError {
    StoreError::Io(e.to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub mailbox_id: [u8; 32],
    pub id: [u8; 16],
    pub ts: u64,
    pub exp: u64,
    /// `None` = doručeno (náhrobek).
    pub ct: Option<Vec<u8>>,
}

/// Veřejný profil uživatele (`spec/PROTOCOL.md` §15.2) a čas, kdy ho node smaže.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileRecord {
    pub pk: [u8; 32],
    pub bk: [u8; 32],
    pub spk: [u8; 32],
    pub otks: Vec<[u8; 32]>,
    pub push_id: String,
    pub ts: u64,
    pub pow: u64,
    pub sig: [u8; 64],
    pub exp: u64,
}

/// Stráž schránky: zápis do ní vyžaduje razítko s `bits` (`spec/PROTOCOL.md` §15.13).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardRecord {
    pub inbox_id: [u8; 32],
    pub pk: [u8; 32],
    pub bits: u8,
    pub ts: u64,
    pub sig: [u8; 64],
    pub exp: u64,
}

fn encode_guard(g: &GuardRecord) -> Vec<u8> {
    let mut v = Vec::with_capacity(GUARD_LEN);
    v.extend_from_slice(&g.pk);
    v.push(g.bits);
    v.extend_from_slice(&g.ts.to_be_bytes());
    v.extend_from_slice(&g.exp.to_be_bytes());
    v.extend_from_slice(&g.sig);
    v
}

fn decode_guard(k: &[u8], v: &[u8]) -> Option<GuardRecord> {
    if k.len() != 32 || v.len() != GUARD_LEN {
        return None;
    }
    Some(GuardRecord {
        inbox_id: k.try_into().ok()?,
        pk: v[..32].try_into().ok()?,
        bits: v[32],
        ts: u64::from_be_bytes(v[33..41].try_into().ok()?),
        exp: u64::from_be_bytes(v[41..49].try_into().ok()?),
        sig: v[49..].try_into().ok()?,
    })
}

/// Stav zprávy z pohledu jednoho anchoru (`spec/PROTOCOL.md` §15.6). Větší hodnota vyhrává při sjednocení.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MsgState {
    Unknown,
    Pending,
    Delivered,
}

impl MsgState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Pending => "pending",
            Self::Delivered => "delivered",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "unknown" => Self::Unknown,
            "pending" => Self::Pending,
            "delivered" => Self::Delivered,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PutOutcome {
    Stored,
    Duplicate,
}

pub struct Store {
    db: Database,
}

fn key(mailbox_id: &[u8; 32], id: &[u8; 16]) -> [u8; KEY_LEN] {
    let mut k = [0u8; KEY_LEN];
    k[..32].copy_from_slice(mailbox_id);
    k[32..].copy_from_slice(id);
    k
}

fn encode(ts: u64, exp: u64, ct: Option<&[u8]>) -> Vec<u8> {
    let mut v = Vec::with_capacity(HEADER_LEN + ct.map_or(0, <[u8]>::len));
    v.push(if ct.is_some() { STATE_PENDING } else { STATE_DELIVERED });
    v.extend_from_slice(&ts.to_be_bytes());
    v.extend_from_slice(&exp.to_be_bytes());
    if let Some(ct) = ct {
        v.extend_from_slice(ct);
    }
    v
}

fn decode(k: &[u8], v: &[u8]) -> Option<Record> {
    if k.len() != KEY_LEN || v.len() < HEADER_LEN {
        return None;
    }
    let ts = u64::from_be_bytes(v[1..9].try_into().ok()?);
    let exp = u64::from_be_bytes(v[9..17].try_into().ok()?);
    Some(Record {
        mailbox_id: k[..32].try_into().ok()?,
        id: k[32..].try_into().ok()?,
        ts,
        exp,
        ct: (v[0] == STATE_PENDING).then(|| v[HEADER_LEN..].to_vec()),
    })
}

fn encode_profile(p: &ProfileRecord) -> Vec<u8> {
    let mut v = Vec::with_capacity(PROFILE_FIXED_LEN + 32 * p.otks.len());
    v.extend_from_slice(&p.ts.to_be_bytes());
    v.extend_from_slice(&p.exp.to_be_bytes());
    v.extend_from_slice(&p.pow.to_be_bytes());
    v.extend_from_slice(&p.bk);
    v.extend_from_slice(&p.spk);
    v.extend_from_slice(&p.sig);
    v.extend_from_slice(p.push_id.as_bytes());
    v.push(u8::try_from(p.otks.len()).unwrap_or(u8::MAX));
    for otk in &p.otks {
        v.extend_from_slice(otk);
    }
    v
}

fn decode_profile(k: &[u8], v: &[u8]) -> Option<ProfileRecord> {
    if k.len() != 32 || v.len() < PROFILE_FIXED_LEN {
        return None;
    }
    let count = usize::from(v[PROFILE_FIXED_LEN - 1]);
    if v.len() != PROFILE_FIXED_LEN + 32 * count {
        return None;
    }
    let otks: Vec<[u8; 32]> = v[PROFILE_FIXED_LEN..].as_chunks::<32>().0.to_vec();
    Some(ProfileRecord {
        pk: k.try_into().ok()?,
        ts: u64::from_be_bytes(v[0..8].try_into().ok()?),
        exp: u64::from_be_bytes(v[8..16].try_into().ok()?),
        pow: u64::from_be_bytes(v[16..24].try_into().ok()?),
        bk: v[24..56].try_into().ok()?,
        spk: v[56..88].try_into().ok()?,
        sig: v[88..152].try_into().ok()?,
        push_id: String::from_utf8(v[152..152 + PUSH_ID_LEN].to_vec()).ok()?,
        otks,
    })
}

fn mailbox_range(mailbox_id: &[u8; 32]) -> (Vec<u8>, Vec<u8>) {
    let mut lo = mailbox_id.to_vec();
    lo.extend_from_slice(&[0; 16]);
    let mut hi = mailbox_id.to_vec();
    hi.extend_from_slice(&[0xff; 16]);
    (lo, hi)
}

fn bump(table: &mut redb::Table<&[u8], u64>, mailbox_id: &[u8; 32], delta: i64) -> Result<(), StoreError> {
    let current = table.get(mailbox_id.as_slice()).map_err(io)?.map_or(0, |v| v.value());
    let next = current.saturating_add_signed(delta);
    if next == 0 {
        table.remove(mailbox_id.as_slice()).map_err(io)?;
    } else {
        table.insert(mailbox_id.as_slice(), next).map_err(io)?;
    }
    Ok(())
}

impl Store {
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let db = Database::create(path).map_err(io)?;
        // tabulky musí existovat i pro čtení z prázdné databáze
        let tx = db.begin_write().map_err(io)?;
        tx.open_table(MESSAGES).map_err(io)?;
        tx.open_table(PENDING).map_err(io)?;
        tx.open_table(PROFILES).map_err(io)?;
        tx.open_table(GUARDS).map_err(io)?;
        tx.commit().map_err(io)?;
        Ok(Self { db })
    }

    /// Idempotentní zápis. `ct = None` zapíše náhrobek doručené zprávy; doručeno vyhrává nad čekající.
    pub fn put(&self, mailbox_id: &[u8; 32], id: &[u8; 16], ts: u64, exp: u64, ct: Option<&[u8]>, max_pending: u64) -> Result<PutOutcome, StoreError> {
        let tx = self.db.begin_write().map_err(io)?;
        let outcome = {
            let mut msgs = tx.open_table(MESSAGES).map_err(io)?;
            let mut pending = tx.open_table(PENDING).map_err(io)?;
            let k = key(mailbox_id, id);
            let existing = msgs.get(k.as_slice()).map_err(io)?.map(|v| v.value().to_vec());
            match existing {
                Some(old) if old.first() == Some(&STATE_DELIVERED) || ct.is_some() => PutOutcome::Duplicate,
                Some(old) => {
                    // čekající záznam se mění na doručený (ct je None)
                    let (ts_old, exp_old) = (old.get(1..9), old.get(9..17));
                    if let (Some(ts_old), Some(exp_old)) = (ts_old, exp_old) {
                        let mut kept = vec![STATE_DELIVERED];
                        kept.extend_from_slice(ts_old);
                        kept.extend_from_slice(exp_old);
                        msgs.insert(k.as_slice(), kept.as_slice()).map_err(io)?;
                        bump(&mut pending, mailbox_id, -1)?;
                    }
                    PutOutcome::Duplicate
                }
                None => {
                    if ct.is_some() {
                        let count = pending.get(mailbox_id.as_slice()).map_err(io)?.map_or(0, |v| v.value());
                        if count >= max_pending {
                            return Err(StoreError::Full);
                        }
                        bump(&mut pending, mailbox_id, 1)?;
                    }
                    msgs.insert(k.as_slice(), encode(ts, exp, ct).as_slice()).map_err(io)?;
                    PutOutcome::Stored
                }
            }
        };
        tx.commit().map_err(io)?;
        Ok(outcome)
    }

    /// Označí zprávy jako doručené (ct se zahodí). Neznámé `id` dostane náhrobek s `exp`. Vrací počet změn.
    pub fn ack(&self, mailbox_id: &[u8; 32], ids: &[[u8; 16]], now: u64, exp: u64) -> Result<usize, StoreError> {
        let mut changed = 0;
        let tx = self.db.begin_write().map_err(io)?;
        {
            let mut msgs = tx.open_table(MESSAGES).map_err(io)?;
            let mut pending = tx.open_table(PENDING).map_err(io)?;
            for id in ids {
                let k = key(mailbox_id, id);
                let old = msgs.get(k.as_slice()).map_err(io)?.map(|v| v.value().to_vec());
                match old {
                    Some(old) if old.first() == Some(&STATE_DELIVERED) => {}
                    Some(old) => {
                        let mut kept = vec![STATE_DELIVERED];
                        kept.extend_from_slice(old.get(1..17).unwrap_or(&[0; 16]));
                        msgs.insert(k.as_slice(), kept.as_slice()).map_err(io)?;
                        bump(&mut pending, mailbox_id, -1)?;
                        changed += 1;
                    }
                    None => {
                        msgs.insert(k.as_slice(), encode(now, exp, None).as_slice()).map_err(io)?;
                        changed += 1;
                    }
                }
            }
        }
        tx.commit().map_err(io)?;
        Ok(changed)
    }

    /// Stav zpráv (`ids`) ve stejném pořadí. Vypršené záznamy se hlásí jako neznámé.
    pub fn states(&self, mailbox_id: &[u8; 32], ids: &[[u8; 16]], now: u64) -> Result<Vec<MsgState>, StoreError> {
        let tx = self.db.begin_read().map_err(io)?;
        let msgs = tx.open_table(MESSAGES).map_err(io)?;
        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            let rec = msgs.get(key(mailbox_id, id).as_slice()).map_err(io)?.and_then(|v| decode(&key(mailbox_id, id), v.value()));
            out.push(match rec {
                Some(r) if r.exp > now => {
                    if r.ct.is_some() {
                        MsgState::Pending
                    } else {
                        MsgState::Delivered
                    }
                }
                _ => MsgState::Unknown,
            });
        }
        Ok(out)
    }

    /// Zapíše profil, je-li novější než ten uložený. `Stored` = nový nebo nahrazený, `Duplicate` = nic se nezměnilo.
    pub fn put_profile(&self, p: &ProfileRecord, max_profiles: u64) -> Result<PutOutcome, StoreError> {
        let tx = self.db.begin_write().map_err(io)?;
        let outcome = {
            let mut table = tx.open_table(PROFILES).map_err(io)?;
            let old = table.get(p.pk.as_slice()).map_err(io)?.and_then(|v| decode_profile(&p.pk, v.value()));
            match old {
                Some(old) if old.ts >= p.ts => PutOutcome::Duplicate,
                Some(_) => {
                    table.insert(p.pk.as_slice(), encode_profile(p).as_slice()).map_err(io)?;
                    PutOutcome::Stored
                }
                None => {
                    if table.len().map_err(io)? >= max_profiles {
                        return Err(StoreError::Full);
                    }
                    table.insert(p.pk.as_slice(), encode_profile(p).as_slice()).map_err(io)?;
                    PutOutcome::Stored
                }
            }
        };
        tx.commit().map_err(io)?;
        Ok(outcome)
    }

    /// Profil uživatele, pokud ho node drží a nevypršel.
    pub fn profile(&self, pk: &[u8; 32], now: u64) -> Result<Option<ProfileRecord>, StoreError> {
        let tx = self.db.begin_read().map_err(io)?;
        let table = tx.open_table(PROFILES).map_err(io)?;
        let rec = table.get(pk.as_slice()).map_err(io)?.and_then(|v| decode_profile(pk, v.value()));
        Ok(rec.filter(|r| r.exp > now))
    }

    /// Všechny nevypršené profily (handoff).
    pub fn profiles(&self, now: u64) -> Result<Vec<ProfileRecord>, StoreError> {
        let tx = self.db.begin_read().map_err(io)?;
        let table = tx.open_table(PROFILES).map_err(io)?;
        let mut out = Vec::new();
        for entry in table.iter().map_err(io)? {
            let (k, v) = entry.map_err(io)?;
            out.extend(decode_profile(k.value(), v.value()).filter(|r| r.exp > now));
        }
        Ok(out)
    }

    /// Smaže profily (po handoffu, kdy node už není anchorem).
    pub fn remove_profiles(&self, pks: &[[u8; 32]]) -> Result<(), StoreError> {
        let tx = self.db.begin_write().map_err(io)?;
        {
            let mut table = tx.open_table(PROFILES).map_err(io)?;
            for pk in pks {
                table.remove(pk.as_slice()).map_err(io)?;
            }
        }
        tx.commit().map_err(io)
    }

    /// Zapíše stráž, je-li novější než uložená (`ts`). Strop `max` počítá stráže dohromady s profily nepřímo: stráž
    /// patří k adrese, takže jich není víc než profilů.
    pub fn put_guard(&self, g: &GuardRecord, max: u64) -> Result<PutOutcome, StoreError> {
        let tx = self.db.begin_write().map_err(io)?;
        let outcome = {
            let mut table = tx.open_table(GUARDS).map_err(io)?;
            let old = table.get(g.inbox_id.as_slice()).map_err(io)?.and_then(|v| decode_guard(&g.inbox_id, v.value()));
            match old {
                Some(old) if old.ts >= g.ts => PutOutcome::Duplicate,
                None if table.len().map_err(io)? >= max => return Err(StoreError::Full),
                _ => {
                    table.insert(g.inbox_id.as_slice(), encode_guard(g).as_slice()).map_err(io)?;
                    PutOutcome::Stored
                }
            }
        };
        tx.commit().map_err(io)?;
        Ok(outcome)
    }

    /// Stráž mailboxu, pokud ji node drží a nevypršela.
    pub fn guard(&self, inbox_id: &[u8; 32], now: u64) -> Result<Option<GuardRecord>, StoreError> {
        let tx = self.db.begin_read().map_err(io)?;
        let table = tx.open_table(GUARDS).map_err(io)?;
        let rec = table.get(inbox_id.as_slice()).map_err(io)?.and_then(|v| decode_guard(inbox_id, v.value()));
        Ok(rec.filter(|r| r.exp > now))
    }

    /// Všechny nevypršené stráže (handoff).
    pub fn guards(&self, now: u64) -> Result<Vec<GuardRecord>, StoreError> {
        let tx = self.db.begin_read().map_err(io)?;
        let table = tx.open_table(GUARDS).map_err(io)?;
        let mut out = Vec::new();
        for entry in table.iter().map_err(io)? {
            let (k, v) = entry.map_err(io)?;
            out.extend(decode_guard(k.value(), v.value()).filter(|r| r.exp > now));
        }
        Ok(out)
    }

    pub fn remove_guards(&self, ids: &[[u8; 32]]) -> Result<(), StoreError> {
        let tx = self.db.begin_write().map_err(io)?;
        {
            let mut table = tx.open_table(GUARDS).map_err(io)?;
            for id in ids {
                table.remove(id.as_slice()).map_err(io)?;
            }
        }
        tx.commit().map_err(io)
    }

    /// Nedoručené a nevypršené zprávy mailboxu seřazené podle `(ts, id)`.
    pub fn pending(&self, mailbox_id: &[u8; 32], limit: usize, now: u64) -> Result<Vec<Record>, StoreError> {
        let mut out: Vec<Record> = self.records(mailbox_id)?.into_iter().filter(|r| r.ct.is_some() && r.exp > now).collect();
        out.sort_by_key(|r| (r.ts, r.id));
        out.truncate(limit);
        Ok(out)
    }

    /// Všechny záznamy mailboxu včetně náhrobků.
    pub fn records(&self, mailbox_id: &[u8; 32]) -> Result<Vec<Record>, StoreError> {
        let tx = self.db.begin_read().map_err(io)?;
        let msgs = tx.open_table(MESSAGES).map_err(io)?;
        let (lo, hi) = mailbox_range(mailbox_id);
        let mut out = Vec::new();
        for entry in msgs.range(lo.as_slice()..=hi.as_slice()).map_err(io)? {
            let (k, v) = entry.map_err(io)?;
            out.extend(decode(k.value(), v.value()));
        }
        Ok(out)
    }

    /// Mailboxy, které mají aspoň jeden záznam.
    pub fn mailboxes(&self) -> Result<Vec<[u8; 32]>, StoreError> {
        let tx = self.db.begin_read().map_err(io)?;
        let msgs = tx.open_table(MESSAGES).map_err(io)?;
        let mut out: Vec<[u8; 32]> = Vec::new();
        for entry in msgs.iter().map_err(io)? {
            let (k, _) = entry.map_err(io)?;
            if let Ok(mb) = <[u8; 32]>::try_from(&k.value()[..32.min(k.value().len())]) {
                if out.last() != Some(&mb) {
                    out.push(mb);
                }
            }
        }
        Ok(out)
    }

    /// Smaže záznamy bez ohledu na stav (handoff, úklid).
    pub fn remove(&self, mailbox_id: &[u8; 32], ids: &[[u8; 16]]) -> Result<(), StoreError> {
        let tx = self.db.begin_write().map_err(io)?;
        {
            let mut msgs = tx.open_table(MESSAGES).map_err(io)?;
            let mut pending = tx.open_table(PENDING).map_err(io)?;
            for id in ids {
                let old = msgs.remove(key(mailbox_id, id).as_slice()).map_err(io)?.map(|v| v.value().first().copied());
                if old == Some(Some(STATE_PENDING)) {
                    bump(&mut pending, mailbox_id, -1)?;
                }
            }
        }
        tx.commit().map_err(io)
    }

    /// Smaže vypršené záznamy. Vrací jejich počet.
    pub fn purge_expired(&self, now: u64) -> Result<usize, StoreError> {
        let expired: Vec<Record> = {
            let tx = self.db.begin_read().map_err(io)?;
            let msgs = tx.open_table(MESSAGES).map_err(io)?;
            let mut v = Vec::new();
            for entry in msgs.iter().map_err(io)? {
                let (k, val) = entry.map_err(io)?;
                if let Some(r) = decode(k.value(), val.value()).filter(|r| r.exp <= now) {
                    v.push(r);
                }
            }
            v
        };
        let mut removed = 0;
        for r in &expired {
            self.remove(&r.mailbox_id, &[r.id])?;
            removed += 1;
        }
        let stale: Vec<[u8; 32]> = {
            let tx = self.db.begin_read().map_err(io)?;
            let table = tx.open_table(PROFILES).map_err(io)?;
            let mut v = Vec::new();
            for entry in table.iter().map_err(io)? {
                let (k, val) = entry.map_err(io)?;
                if let Some(p) = decode_profile(k.value(), val.value()).filter(|p| p.exp <= now) {
                    v.push(p.pk);
                }
            }
            v
        };
        self.remove_profiles(&stale)?;
        removed += stale.len();
        let now_guards: Vec<[u8; 32]> = {
            let tx = self.db.begin_read().map_err(io)?;
            let table = tx.open_table(GUARDS).map_err(io)?;
            let mut v = Vec::new();
            for entry in table.iter().map_err(io)? {
                let (k, val) = entry.map_err(io)?;
                if let Some(g) = decode_guard(k.value(), val.value()).filter(|g| g.exp <= now) {
                    v.push(g.inbox_id);
                }
            }
            v
        };
        self.remove_guards(&now_guards)?;
        removed += now_guards.len();
        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let s = Store::open(&dir.path().join("t.redb")).unwrap();
        (dir, s)
    }

    const MB: [u8; 32] = [1; 32];

    #[test]
    fn put_is_idempotent_and_sorted() {
        let (_d, s) = store();
        assert_eq!(s.put(&MB, &[2; 16], 20, 1000, Some(b"b"), 10).unwrap(), PutOutcome::Stored);
        assert_eq!(s.put(&MB, &[1; 16], 10, 1000, Some(b"a"), 10).unwrap(), PutOutcome::Stored);
        assert_eq!(s.put(&MB, &[1; 16], 10, 1000, Some(b"different"), 10).unwrap(), PutOutcome::Duplicate);
        let got = s.pending(&MB, 10, 0).unwrap();
        assert_eq!(got.iter().map(|r| r.ct.clone().unwrap()).collect::<Vec<_>>(), vec![b"a".to_vec(), b"b".to_vec()]);
        assert_eq!(s.pending(&MB, 1, 0).unwrap().len(), 1);
        assert!(s.pending(&[9; 32], 10, 0).unwrap().is_empty());
    }

    #[test]
    fn ack_leaves_tombstone_that_blocks_late_put() {
        let (_d, s) = store();
        s.put(&MB, &[1; 16], 10, 1000, Some(b"a"), 10).unwrap();
        assert_eq!(s.ack(&MB, &[[1; 16], [7; 16]], 50, 500).unwrap(), 2);
        assert_eq!(s.ack(&MB, &[[1; 16]], 50, 500).unwrap(), 0);
        assert!(s.pending(&MB, 10, 0).unwrap().is_empty());
        assert_eq!(s.put(&MB, &[7; 16], 10, 1000, Some(b"late"), 10).unwrap(), PutOutcome::Duplicate);
        assert!(s.pending(&MB, 10, 0).unwrap().is_empty());
        assert_eq!(s.records(&MB).unwrap().len(), 2);
    }

    #[test]
    fn delivered_put_upgrades_pending() {
        let (_d, s) = store();
        s.put(&MB, &[1; 16], 10, 1000, Some(b"a"), 10).unwrap();
        assert_eq!(s.put(&MB, &[1; 16], 10, 1000, None, 10).unwrap(), PutOutcome::Duplicate);
        assert!(s.pending(&MB, 10, 0).unwrap().is_empty());
    }

    #[test]
    fn mailbox_cap_counts_only_pending() {
        let (_d, s) = store();
        s.put(&MB, &[1; 16], 1, 1000, Some(b"a"), 2).unwrap();
        s.put(&MB, &[2; 16], 1, 1000, Some(b"b"), 2).unwrap();
        assert_eq!(s.put(&MB, &[3; 16], 1, 1000, Some(b"c"), 2), Err(StoreError::Full));
        s.ack(&MB, &[[1; 16]], 5, 500).unwrap();
        assert_eq!(s.put(&MB, &[3; 16], 1, 1000, Some(b"c"), 2).unwrap(), PutOutcome::Stored);
    }

    #[test]
    fn purge_and_listing() {
        let (_d, s) = store();
        s.put(&MB, &[1; 16], 1, 100, Some(b"old"), 10).unwrap();
        s.put(&MB, &[2; 16], 1, 900, Some(b"new"), 10).unwrap();
        s.put(&[5; 32], &[1; 16], 1, 100, Some(b"other"), 10).unwrap();
        assert_eq!(s.mailboxes().unwrap().len(), 2);
        assert_eq!(s.purge_expired(500).unwrap(), 2);
        assert_eq!(s.mailboxes().unwrap(), vec![MB]);
        assert_eq!(s.pending(&MB, 10, 500).unwrap().len(), 1);
        // expirovaná, ale ještě neuklizená zpráva se nevrací
        assert!(s.pending(&MB, 10, 1000).unwrap().is_empty());
    }

    #[test]
    fn survives_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.redb");
        {
            let s = Store::open(&path).unwrap();
            s.put(&MB, &[1; 16], 1, 1000, Some(b"keep"), 10).unwrap();
        }
        let s = Store::open(&path).unwrap();
        assert_eq!(s.pending(&MB, 10, 0).unwrap().len(), 1);
    }
}
