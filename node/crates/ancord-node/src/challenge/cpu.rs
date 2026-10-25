//! CPU challenge: časovaný sekvenční hash-chain (`spec/PROTOCOL.md` §9.1).

use sha2::{Digest, Sha256};

use crate::encoding::b64u_encode;

#[derive(Debug, thiserror::Error)]
pub enum CpuError {
    #[error("invalid cpu challenge parameters")]
    BadParams,
}

fn lane_start(seed: &[u8; 32], lane: u32) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(seed);
    h.update(lane.to_le_bytes());
    h.finalize().into()
}

fn lane_checkpoints(seed: &[u8; 32], lane: u32, iters: u64, every: u64) -> Vec<String> {
    let mut h = lane_start(seed, lane);
    let mut out = Vec::with_capacity((iters / every) as usize);
    for _ in 0..iters / every {
        for _ in 0..every {
            h = Sha256::digest(h).into();
        }
        out.push(b64u_encode(&h));
    }
    out
}

/// Spočte checkpointy všech drah; dráhy běží paralelně na samostatných vláknech.
pub fn compute_checkpoints(seed: &[u8; 32], lanes: u32, iters: u64, every: u64) -> Result<Vec<Vec<String>>, CpuError> {
    if lanes == 0 || every == 0 || iters == 0 || !iters.is_multiple_of(every) {
        return Err(CpuError::BadParams);
    }
    Ok(std::thread::scope(|scope| {
        let handles: Vec<_> = (0..lanes).map(|lane| scope.spawn(move || lane_checkpoints(seed, lane, iters, every))).collect();
        handles.into_iter().map(|h| h.join().unwrap_or_default()).collect()
    }))
}
