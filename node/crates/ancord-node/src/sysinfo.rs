//! Měření hostitele: tvrzené parametry při registraci a zátěž v heartbeatu.

use std::path::Path;
use std::sync::Mutex;

use sysinfo::{Disks, System};

#[derive(Debug, Clone, Copy)]
pub struct Measured {
    pub cpu_cores: u64,
    pub ram_mb: u64,
    pub storage_gb: u64,
}

pub fn measure(data_dir: &Path) -> Measured {
    let mut sys = System::new();
    sys.refresh_memory();
    let cores = std::thread::available_parallelism().map_or(1, |n| n.get() as u64);
    let dir = std::fs::canonicalize(data_dir).unwrap_or_else(|_| data_dir.to_path_buf());
    let disks = Disks::new_with_refreshed_list();
    let free = disks
        .list()
        .iter()
        .filter(|d| dir.starts_with(d.mount_point()))
        .max_by_key(|d| d.mount_point().as_os_str().len())
        .map_or(1, |d| d.available_space() / (1 << 30));
    Measured { cpu_cores: cores, ram_mb: (sys.total_memory() / (1 << 20)).max(1), storage_gb: free.max(1) }
}

/// Sledování zátěže procesoru a paměti pro heartbeat.
pub struct LoadMeter {
    sys: Mutex<System>,
}

impl Default for LoadMeter {
    fn default() -> Self {
        Self { sys: Mutex::new(System::new()) }
    }
}

impl LoadMeter {
    /// `(cpu_pct, mem_pct)` v rozsahu 0–100.
    pub fn sample(&self) -> (u64, u64) {
        let Ok(mut sys) = self.sys.lock() else {
            return (0, 0);
        };
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        let cpu = f64::from(sys.global_cpu_usage()).round().clamp(0.0, 100.0) as u64;
        let total = sys.total_memory().max(1);
        let mem = (sys.used_memory() as f64 / total as f64 * 100.0).round().clamp(0.0, 100.0) as u64;
        (cpu, mem)
    }
}
