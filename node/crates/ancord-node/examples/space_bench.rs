//! Měření Proof of Space (`ARCHITECTURE.md` §9 bod 3): stavba tabulky, latence otevření a cena útoku s částečnou tabulkou.
//!
//! ```text
//! cargo run --release -p ancord-node --example space_bench -- build <log2 leaves> <ram|storage> [dir]
//! cargo run --release -p ancord-node --example space_bench -- attack <log2 leaves> <stride>...
//! ```
//! `build` po stavbě vypíše cesty k souborům a čeká na řádek ze stdin; mezitím jde stránky vyhodit z page cache
//! (`posix_fadvise(DONTNEED)`), a další otevření se tak měří ze studeného disku.

use std::collections::HashMap;
use std::io::BufRead;
use std::path::PathBuf;
use std::time::Instant;

use ancord_node::challenge::space::SpaceTable;
use rand::Rng;
use sha2::{Digest, Sha256};

const OPENINGS: u64 = 16;
const CHAIN: u64 = 64;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("build") => build(&args[1..]),
        Some("attack") => attack(&args[1..]),
        _ => eprintln!("usage: space_bench build <log2> <ram|storage> [dir] | attack <log2> <stride>..."),
    }
}

fn log2_arg(a: Option<&String>) -> u64 {
    1u64 << a.and_then(|s| s.parse::<u32>().ok()).unwrap_or(16)
}

fn open_round(t: &SpaceTable, rng: &mut impl Rng) -> f64 {
    let started = Instant::now();
    for _ in 0..OPENINGS {
        let i = rng.gen_range(1..t.leaves() as u64);
        if t.opening(i).is_err() {
            eprintln!("opening {i} failed");
        }
    }
    started.elapsed().as_secs_f64() * 1000.0
}

fn report(label: &str, mut ms: Vec<f64>) {
    ms.sort_by(f64::total_cmp);
    let p = |q: f64| ms[((ms.len() - 1) as f64 * q) as usize];
    println!("{label}: {} kol × {OPENINGS} otevření, medián {:.2} ms, p90 {:.2} ms, max {:.2} ms", ms.len(), p(0.5), p(0.9), p(1.0));
}

fn build(args: &[String]) {
    let leaves = log2_arg(args.first());
    let storage = args.get(1).is_some_and(|s| s == "storage");
    let dir = PathBuf::from(args.get(2).cloned().unwrap_or_else(|| std::env::temp_dir().display().to_string()));
    let seed: [u8; 32] = rand::random();
    let started = Instant::now();
    let backing = storage.then_some((dir.as_path(), "bench"));
    let t = match SpaceTable::build(seed, leaves, backing) {
        Ok(t) => t,
        Err(e) => return eprintln!("build failed: {e}"),
    };
    println!(
        "stavba: {} listů ({} MiB tabulka + strom), {}, {:.1} s",
        leaves,
        leaves * 32 / (1 << 20),
        if storage { "soubor" } else { "RAM" },
        started.elapsed().as_secs_f64()
    );
    let mut rng = rand::thread_rng();
    report("teplé otevření", (0..50).map(|_| open_round(&t, &mut rng)).collect());
    if storage {
        println!("files: {}/space-bench.table {}/space-bench.tree", dir.display(), dir.display());
        println!("čekám na řádek (vyhoď page cache)…");
        let mut line = String::new();
        let _ = std::io::stdin().lock().read_line(&mut line);
        // každé kolo jen jednou: další by už četlo z cache
        report("studené otevření (první kolo)", vec![open_round(&t, &mut rng)]);
    }
}

/* ------------------------------------------------------------------ útok */

fn t0(seed: &[u8; 32]) -> [u8; 32] {
    Sha256::new().chain_update(b"ancord/space/0").chain_update(seed).finalize().into()
}

/// `T[i]` s `refs` odkazy (`refs = 1` je dnešní §9.2).
fn ti(seed: &[u8; 32], i: u64, prev: &[u8; 32], refs: &[[u8; 32]]) -> [u8; 32] {
    let mut h = Sha256::new().chain_update(b"ancord/space/i").chain_update(seed).chain_update(i.to_le_bytes()).chain_update(prev);
    for r in refs {
        h.update(r);
    }
    h.finalize().into()
}

/// `j_m(i) = u64_le(T[i−1][8m..8m+8]) mod i`.
fn ref_indices(prev: &[u8; 32], i: u64, d: usize) -> Vec<u64> {
    (0..d)
        .map(|m| {
            let mut b = [0u8; 8];
            b.copy_from_slice(&prev[8 * m..8 * m + 8]);
            u64::from_le_bytes(b) % i
        })
        .collect()
}

/// Útočník drží jen `T[k]` pro `k % stride == 0`; chybějící bloky dopočítá (memo jen v rámci jedné výzvy).
struct Attacker<'a> {
    seed: [u8; 32],
    table: &'a [[u8; 32]],
    stride: u64,
    d: usize,
    memo: HashMap<u64, [u8; 32]>,
    hashes: u64,
    budget: u64,
}

impl Attacker<'_> {
    fn lookup(&self, k: u64) -> Option<[u8; 32]> {
        if k.is_multiple_of(self.stride) { Some(self.table[k as usize]) } else { self.memo.get(&k).copied() }
    }

    /// `None`, když útočník vyčerpá rozpočet hashů.
    fn get(&mut self, k: u64) -> Option<[u8; 32]> {
        // explicitní zásobník: rekurze by u řídkých tabulek přetekla
        let mut stack = vec![k];
        while let Some(&top) = stack.last() {
            if self.lookup(top).is_some() {
                stack.pop();
                continue;
            }
            let Some(prev) = self.lookup(top - 1) else {
                stack.push(top - 1);
                continue;
            };
            let idx = ref_indices(&prev, top, self.d);
            if let Some(&missing) = idx.iter().find(|&&j| self.lookup(j).is_none()) {
                stack.push(missing);
                continue;
            }
            let refs: Vec<[u8; 32]> = idx.iter().filter_map(|&j| self.lookup(j)).collect();
            let v = ti(&self.seed, top, &prev, &refs);
            self.hashes += 1;
            if self.hashes > self.budget {
                return None;
            }
            self.memo.insert(top, v);
            stack.pop();
        }
        self.lookup(k)
    }
}

/// `attack <log2> <refs> <stride>...`
fn attack(args: &[String]) {
    let leaves = log2_arg(args.first());
    let d: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1).clamp(1, 4);
    let seed: [u8; 32] = rand::random();
    let mut table: Vec<[u8; 32]> = Vec::with_capacity(leaves as usize);
    table.push(t0(&seed));
    let started = Instant::now();
    for i in 1..leaves {
        let prev = table[(i - 1) as usize];
        let refs: Vec<[u8; 32]> = ref_indices(&prev, i, d).iter().map(|&j| table[j as usize]).collect();
        table.push(ti(&seed, i, &prev, &refs));
    }
    let secs = started.elapsed().as_secs_f64();
    println!("refs={d}, tabulka {leaves} listů: {secs:.2} s sekvenčně, {:.2} M hashů/s", leaves as f64 / secs / 1e6);
    // rozpočet = celá tabulka znovu; víc nemá smysl (to je přestavba)
    let budget = leaves;
    let mut rng = rand::thread_rng();
    for s in &args[2..] {
        let Ok(stride) = s.parse::<u64>() else { continue };
        let mut costs = Vec::new();
        for _ in 0..10 {
            let mut worst = 0;
            for _ in 0..OPENINGS {
                // otevření jsou nezávislá (paralelní útočník), memo se nesdílí
                let mut a = Attacker { seed, table: &table, stride, d, memo: HashMap::new(), hashes: 0, budget };
                let i = rng.gen_range(1..leaves);
                let ok = a.get(i - 1).is_some_and(|prev| {
                    a.get(i).is_some() && ref_indices(&prev, i, d).iter().all(|&j| a.get(j).is_some())
                });
                worst = worst.max(if ok { a.hashes } else { u64::MAX });
            }
            costs.push(worst);
        }
        // řetězená otevření: index dalšího se odvodí z předchozího listu, útočník je počítá postupně se sdíleným memem
        let mut chain = Vec::new();
        for _ in 0..5 {
            let mut a = Attacker { seed, table: &table, stride, d, memo: HashMap::new(), hashes: 0, budget: u64::MAX };
            let mut i = rng.gen_range(1..leaves);
            for _ in 0..CHAIN {
                let prev = a.get(i - 1).unwrap_or_default();
                for j in ref_indices(&prev, i, d) {
                    a.get(j);
                }
                let leaf = a.get(i).unwrap_or_default();
                i = 1 + u64::from_le_bytes(leaf[..8].try_into().unwrap_or_default()) % (leaves - 1);
                if a.hashes > 50 * leaves {
                    break;
                }
            }
            chain.push(a.hashes);
        }
        chain.sort_unstable();
        println!("  uloženo 1/{stride}: řetěz {CHAIN} otevření sekvenčně, hashů medián {}, max {}", chain[chain.len() / 2], chain[chain.len() - 1]);
        costs.sort_unstable();
        let show = |v: u64| if v == u64::MAX { format!(">{budget}") } else { v.to_string() };
        println!(
            "  uloženo 1/{stride}: nejdražší z {OPENINGS} otevření (paralelně) medián {}, max {}",
            show(costs[costs.len() / 2]),
            show(costs[costs.len() - 1])
        );
    }
}
