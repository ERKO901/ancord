//! Spuštění `cloudflared` s tokenem od Workeru (`ARCHITECTURE.md` §4). Token se předává proměnnou prostředí,
//! ne argumentem, aby nebyl vidět v seznamu procesů.

use std::time::Duration;

use tokio::process::{Child, Command};
use tokio::task::JoinHandle;

pub struct Tunnel {
    task: JoinHandle<()>,
}

impl Tunnel {
    /// Spustí `cloudflared` a při pádu ho po krátké pauze znovu nastartuje.
    pub fn spawn(bin: String, token: String) -> Self {
        let task = tokio::spawn(async move {
            let mut attempt = 0u32;
            loop {
                match start(&bin, &token) {
                    Ok(mut child) => {
                        tracing::info!("cloudflared started");
                        match child.wait().await {
                            Ok(status) => tracing::warn!(%status, "cloudflared exited"),
                            Err(e) => tracing::warn!(error = %e, "cloudflared wait failed"),
                        }
                    }
                    Err(e) => tracing::error!(error = %e, bin = %bin, "cannot start cloudflared"),
                }
                attempt += 1;
                tokio::time::sleep(Duration::from_secs(u64::from(attempt.min(6)) * 5)).await;
            }
        });
        Self { task }
    }
}

fn start(bin: &str, token: &str) -> std::io::Result<Child> {
    Command::new(bin).args(["tunnel", "--no-autoupdate", "run"]).env("TUNNEL_TOKEN", token).kill_on_drop(true).spawn()
}

impl Drop for Tunnel {
    fn drop(&mut self) {
        self.task.abort();
    }
}
