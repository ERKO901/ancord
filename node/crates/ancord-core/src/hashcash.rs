//! Hashcash při registraci (`spec/PROTOCOL.md` §4.2): `sha256(nonce ‖ pubkey ‖ counter_le_u64)`.

use std::sync::atomic::{AtomicU64, Ordering};

use sha2::{Digest, Sha256};

fn leading_zero_bits(bytes: &[u8]) -> u32 {
    let mut n = 0;
    for b in bytes {
        if *b == 0 {
            n += 8;
        } else {
            return n + b.leading_zeros();
        }
    }
    n
}

pub fn valid(nonce: &[u8; 32], pubkey: &[u8; 32], counter: u64, bits: u32) -> bool {
    if bits == 0 {
        return true;
    }
    let mut h = Sha256::new();
    h.update(nonce);
    h.update(pubkey);
    h.update(counter.to_le_bytes());
    leading_zero_bits(&h.finalize()) >= bits
}

/// Najde nejmenší platný `counter` (vlákna procházejí prokládaně).
pub fn solve(nonce: &[u8; 32], pubkey: &[u8; 32], bits: u32, threads: usize) -> u64 {
    if bits == 0 {
        return 0;
    }
    let threads = threads.max(1) as u64;
    let best = AtomicU64::new(u64::MAX);
    std::thread::scope(|scope| {
        for first in 0..threads {
            let best = &best;
            scope.spawn(move || {
                let mut c = first;
                while c < best.load(Ordering::Relaxed) {
                    if valid(nonce, pubkey, c, bits) {
                        best.fetch_min(c, Ordering::Relaxed);
                        return;
                    }
                    c += threads;
                }
            });
        }
    });
    best.load(Ordering::Relaxed)
}
