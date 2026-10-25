//! Proof of Space s Merkle commitmentem (`spec/PROTOCOL.md` §9.2).
//!
//! Tabulka i strom leží buď v paměti (tier `ram`), nebo v souboru (tier `storage`): právě to je to, co se prokazuje.

use std::fs::{File, OpenOptions};
use std::io;
use std::os::unix::fs::FileExt;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::encoding::b64u_encode;

pub const LEAF_BYTES: usize = 32;
const D0: &[u8] = b"ancord/space/0";
const DI: &[u8] = b"ancord/space/i";
const CHUNK: usize = 1 << 16;
const PAIR: usize = 2 * LEAF_BYTES;

#[derive(Debug, thiserror::Error)]
pub enum SpaceError {
    #[error("leaves must be a power of two >= 2")]
    BadLeaves,
    #[error("index {0} out of range")]
    BadIndex(u64),
    #[error("io: {0}")]
    Io(#[from] io::Error),
}

fn t0(seed: &[u8; 32]) -> [u8; 32] {
    Sha256::new().chain_update(D0).chain_update(seed).finalize().into()
}

fn ti(seed: &[u8; 32], i: u64, prev: &[u8], reference: &[u8]) -> [u8; 32] {
    Sha256::new()
        .chain_update(DI)
        .chain_update(seed)
        .chain_update(i.to_le_bytes())
        .chain_update(prev)
        .chain_update(reference)
        .finalize()
        .into()
}

fn leaf_hash(t: &[u8]) -> [u8; 32] {
    Sha256::new().chain_update([0u8]).chain_update(t).finalize().into()
}

fn node_hash(l: &[u8], r: &[u8]) -> [u8; 32] {
    Sha256::new().chain_update([1u8]).chain_update(l).chain_update(r).finalize().into()
}

/// `j(i) = u64_le(T[i−1][0..8]) mod i`.
fn ref_index(prev: &[u8], i: u64) -> u64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&prev[..8]);
    u64::from_le_bytes(b) % i
}

/// Úložiště bajtů s náhodným přístupem.
enum Backing {
    Mem(Vec<u8>),
    File(File),
}

impl Backing {
    fn create(len: u64, path: Option<&Path>) -> io::Result<Self> {
        match path {
            None => Ok(Self::Mem(vec![0; len as usize])),
            Some(p) => {
                let f = OpenOptions::new().read(true).write(true).create(true).truncate(true).open(p)?;
                f.set_len(len)?;
                Ok(Self::File(f))
            }
        }
    }

    fn read_at(&self, off: u64, buf: &mut [u8]) -> io::Result<()> {
        match self {
            Self::Mem(v) => {
                let o = off as usize;
                buf.copy_from_slice(&v[o..o + buf.len()]);
                Ok(())
            }
            Self::File(f) => f.read_exact_at(buf, off),
        }
    }

    fn write_at(&mut self, off: u64, data: &[u8]) -> io::Result<()> {
        match self {
            Self::Mem(v) => {
                let o = off as usize;
                v[o..o + data.len()].copy_from_slice(data);
                Ok(())
            }
            Self::File(f) => f.write_all_at(data, off),
        }
    }
}

/// Tabulka a Merkle strom jednoho commitmentu.
pub struct SpaceTable {
    leaves: usize,
    table: Backing,
    /// Strom jako souvislé pole uzlů: úroveň 0 (hashe listů), pak 1, … poslední je kořen.
    tree: Backing,
    root: [u8; 32],
    files: Vec<PathBuf>,
}

/// Odsazení úrovní 1…log2(n) ve stromu (v uzlech). Úroveň 0 (hashe listů) se neukládá, dopočítá se z tabulky.
fn tree_offsets(leaves: usize) -> Vec<usize> {
    let mut offs = Vec::new();
    let (mut off, mut n) = (0, leaves / 2);
    while n >= 1 {
        offs.push(off);
        off += n;
        n /= 2;
    }
    offs
}

impl SpaceTable {
    /// `backing = None` drží vše v paměti, jinak v souborech `space-<name>.{table,tree}` v daném adresáři.
    pub fn build(seed: [u8; 32], leaves: u64, backing: Option<(&Path, &str)>) -> Result<Self, SpaceError> {
        if leaves < 2 || !leaves.is_power_of_two() {
            return Err(SpaceError::BadLeaves);
        }
        let n = leaves as usize;
        let offs = tree_offsets(n);
        let tree_nodes = n - 1;
        let (table_path, tree_path) = match backing {
            Some((dir, name)) => (Some(dir.join(format!("space-{name}.table"))), Some(dir.join(format!("space-{name}.tree")))),
            None => (None, None),
        };
        let mut table = Backing::create((n * LEAF_BYTES) as u64, table_path.as_deref())?;
        let mut tree = Backing::create((tree_nodes * LEAF_BYTES) as u64, tree_path.as_deref())?;

        // T[i] závisí na T[i−1] a T[j(i)]; zápisy se dávkují, odkaz do nezapsané dávky se čte z ní
        let mut pending: Vec<u8> = Vec::with_capacity(CHUNK);
        let mut flushed: usize = 0;
        let mut prev = t0(&seed);
        pending.extend_from_slice(&prev);
        for i in 1..n {
            let j = ref_index(&prev, i as u64) as usize;
            let mut reference = [0u8; LEAF_BYTES];
            if j >= flushed {
                let o = (j - flushed) * LEAF_BYTES;
                reference.copy_from_slice(&pending[o..o + LEAF_BYTES]);
            } else {
                table.read_at((j * LEAF_BYTES) as u64, &mut reference)?;
            }
            prev = ti(&seed, i as u64, &prev, &reference);
            pending.extend_from_slice(&prev);
            if pending.len() >= CHUNK {
                table.write_at((flushed * LEAF_BYTES) as u64, &pending)?;
                flushed += pending.len() / LEAF_BYTES;
                pending.clear();
            }
        }
        if !pending.is_empty() {
            table.write_at((flushed * LEAF_BYTES) as u64, &pending)?;
        }

        // úroveň 1 z tabulky, další úrovně z předchozí; čtení i zápis po blocích
        let mut buf = vec![0u8; CHUNK];
        let mut out = Vec::with_capacity(CHUNK);
        let mut done = 0;
        while done < n / 2 {
            let count = (CHUNK / (2 * LEAF_BYTES)).min(n / 2 - done);
            let slice = &mut buf[..count * 2 * LEAF_BYTES];
            table.read_at((2 * done * LEAF_BYTES) as u64, slice)?;
            out.clear();
            for pair in slice.as_chunks::<PAIR>().0.iter() {
                out.extend_from_slice(&node_hash(&leaf_hash(&pair[..LEAF_BYTES]), &leaf_hash(&pair[LEAF_BYTES..])));
            }
            tree.write_at(((offs[0] + done) * LEAF_BYTES) as u64, &out)?;
            done += count;
        }
        for k in 1..offs.len() {
            let width = n >> (k + 1);
            let mut done = 0;
            while done < width {
                let count = (CHUNK / (2 * LEAF_BYTES)).min(width - done);
                let slice = &mut buf[..count * 2 * LEAF_BYTES];
                tree.read_at(((offs[k - 1] + 2 * done) * LEAF_BYTES) as u64, slice)?;
                out.clear();
                for pair in slice.as_chunks::<PAIR>().0.iter() {
                    out.extend_from_slice(&node_hash(&pair[..LEAF_BYTES], &pair[LEAF_BYTES..]));
                }
                tree.write_at(((offs[k] + done) * LEAF_BYTES) as u64, &out)?;
                done += count;
            }
        }
        let mut root = [0u8; 32];
        tree.read_at((offs[offs.len() - 1] * LEAF_BYTES) as u64, &mut root)?;
        let files = table_path.into_iter().chain(tree_path).collect();
        Ok(Self { leaves: n, table, tree, root, files })
    }

    pub fn leaves(&self) -> usize {
        self.leaves
    }

    pub fn root(&self) -> [u8; 32] {
        self.root
    }

    fn leaf(&self, k: usize) -> Result<[u8; 32], SpaceError> {
        let mut b = [0u8; LEAF_BYTES];
        self.table.read_at((k * LEAF_BYTES) as u64, &mut b)?;
        Ok(b)
    }

    fn path(&self, index: usize) -> Result<Vec<String>, SpaceError> {
        let offs = tree_offsets(self.leaves);
        let mut out = Vec::with_capacity(offs.len() + 1);
        // úroveň 0: sourozenecký list se hashuje z tabulky
        out.push(b64u_encode(&leaf_hash(&self.leaf(index ^ 1)?)));
        let mut idx = index >> 1;
        // poslední uložená úroveň je kořen, ten do cesty nepatří
        for off in &offs[..offs.len() - 1] {
            let mut b = [0u8; 32];
            self.tree.read_at(((off + (idx ^ 1)) * LEAF_BYTES) as u64, &mut b)?;
            out.push(b64u_encode(&b));
            idx >>= 1;
        }
        Ok(out)
    }

    /// Otevření indexu `i` (≥ 1): list, předchozí list a odkazovaný list s cestami.
    pub fn opening(&self, i: u64) -> Result<serde_json::Value, SpaceError> {
        if i == 0 || i >= self.leaves as u64 {
            return Err(SpaceError::BadIndex(i));
        }
        let iu = i as usize;
        let prev = self.leaf(iu - 1)?;
        let j = ref_index(&prev, i) as usize;
        Ok(serde_json::json!({
            "i": i,
            "leaf": b64u_encode(&self.leaf(iu)?),
            "path": self.path(iu)?,
            "prev": b64u_encode(&prev),
            "prev_path": self.path(iu - 1)?,
            "ref": b64u_encode(&self.leaf(j)?),
            "ref_path": self.path(j)?,
        }))
    }
}

impl Drop for SpaceTable {
    fn drop(&mut self) {
        for f in &self.files {
            let _ = std::fs::remove_file(f);
        }
    }
}
