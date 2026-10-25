//! Úložiště mailboxů na anchoru (redb, `spec/PROTOCOL.md` §13.5). Ukládá jen ciphertext a náhrobky doručených zpráv.

use std::fmt::Display;
use std::path::Path;

use redb::{Database, ReadableTable, TableDefinition};

/// Klíč `mailbox_id(32) ‖ id(16)`, hodnota `state(1) ‖ ts_be_u64 ‖ exp_be_u64 ‖ ct`.
const MESSAGES: TableDefinition<&[u8], &[u8]> = TableDefinition::new("messages");
/// Počet nedoručených zpráv v mailboxu (pro strop), klíč `mailbox_id`.
const PENDING: TableDefinition<&[u8], u64> = TableDefinition::new("pending");

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
