//! Konfigurace nodu. Jediný povinný vstup je `WORKER_URL`; vše ostatní má výchozí hodnotu nebo se zjistí z hostitele.
//! Parametry sítě (R, kvorum, epocha, intervaly) node nezná, dostává je od Workeru v adresáři a v `welcome`.

use std::net::SocketAddr;
use std::path::PathBuf;

use clap::Parser;
use serde::Serialize;

#[derive(Debug, Clone, Parser)]
#[command(name = "ancord-node", about = "ancord node")]
pub struct Config {
    /// Adresa Workeru (např. https://ancord.melonet.cz).
    #[arg(long, env = "WORKER_URL")]
    pub worker_url: String,

    /// Adresář s klíčem a daty nodu (perzistentní volume).
    #[arg(long, env = "DATA_DIR", default_value = "./data")]
    pub data_dir: PathBuf,

    /// Jméno do logů.
    #[arg(long, env = "NODE_NAME", default_value = "node")]
    pub node_name: String,

    /// Lokální adresa, na kterou míří ingress tunelu (Worker ji zná jako `NODE_ORIGIN`).
    #[arg(long, env = "LISTEN", default_value = "0.0.0.0:8080")]
    pub listen: SocketAddr,

    /// Veřejný klíč Workeru (b64u). Bez něj se při prvním startu stáhne a uloží (trust on first use).
    #[arg(long, env = "WORKER_PUBKEY")]
    pub worker_pubkey: Option<String>,

    /// Dovolí stáhnout klíč Workeru při prvním startu bez `WORKER_PUBKEY`. Stažení vede přes Cloudflare, takže kdo ho
    /// podvrhne, ovládne důvěru nodu v adresář. V produkci vypnuto: klíč se zadává pevně (`WORKER_PUBKEY`).
    #[arg(long, env = "TRUST_WORKER_KEY_ON_FIRST_USE", action = clap::ArgAction::SetTrue, value_parser = clap::builder::BoolishValueParser::new())]
    pub trust_worker_key_on_first_use: bool,

    /// Příkaz `cloudflared`; hodnota `none` tunel nespouští (lokální vývoj, testy).
    #[arg(long, env = "CLOUDFLARED_BIN", default_value = "cloudflared")]
    pub cloudflared_bin: String,

    /// Tvrzené parametry: výchozí je měření hostitele.
    #[arg(long, env = "CLAIM_CPU_CORES")]
    pub claim_cpu_cores: Option<u64>,
    #[arg(long, env = "CLAIM_RAM_MB")]
    pub claim_ram_mb: Option<u64>,
    #[arg(long, env = "CLAIM_STORAGE_GB")]
    pub claim_storage_gb: Option<u64>,
    #[arg(long, env = "CLAIM_BANDWIDTH_MBPS", default_value_t = 100)]
    pub claim_bandwidth_mbps: u64,

    /// Horní mez spojení hlášená v heartbeatu.
    #[arg(long, env = "MAX_CONNS", default_value_t = 2000)]
    pub max_conns: u64,

    /// Za jak dlouho po upgradu musí klient poslat `auth` (spec §8.1).
    #[arg(long, env = "AUTH_TIMEOUT_S", default_value_t = 10)]
    pub auth_timeout_s: u64,

    /// Strop počtu listů Proof of Space, které node ochotně vytvoří (ochrana před přehnanou výzvou).
    #[arg(long, env = "MAX_SPACE_LEAVES", default_value_t = 1 << 27)]
    pub max_space_leaves: u64,

    /// Šablona adresy jiného nodu pro node↔node requesty; `{host}` se nahradí hostname z adresáře.
    #[arg(long, env = "PEER_URL_TEMPLATE", default_value = "https://{host}")]
    pub peer_url_template: String,

    /// Timeout jednoho node↔node requestu.
    #[arg(long, env = "PEER_TIMEOUT_S", default_value_t = 5)]
    pub peer_timeout_s: u64,

    /// Jak dlouho se uchovává zpráva, která nebyla doručena (spec §13.5).
    #[arg(long, env = "MESSAGE_TTL_S", default_value_t = 30 * 24 * 3600)]
    pub message_ttl_s: u64,

    /// Strop nedoručených zpráv v jednom mailboxu na tomto nodu.
    #[arg(long, env = "MAX_MAILBOX_MESSAGES", default_value_t = 10_000)]
    pub max_mailbox_messages: u64,

    /// Kolik zpráv za sekundu smí poslat jedno spojení klienta (okamžitá dávka je dvojnásobek).
    #[arg(long, env = "CLIENT_MSG_RATE", default_value_t = 20)]
    pub client_msg_rate: u64,

    /// Nejkratší odstup mezi dvěma wake téhož `push_id` z tohoto nodu (spec §13.6).
    #[arg(long, env = "WAKE_MIN_INTERVAL_S", default_value_t = 10)]
    pub wake_min_interval_s: u64,

    /// Jak dlouho platí presence mailboxu na anchoru, když ji gateway neobnoví (spec §13.7).
    #[arg(long, env = "PRESENCE_TTL_S", default_value_t = 60)]
    pub presence_ttl_s: u64,

    /// Jak často gateway obnovuje presence svých odběratelů (méně než polovina TTL).
    #[arg(long, env = "PRESENCE_RENEW_S", default_value_t = 20)]
    pub presence_renew_s: u64,

    /// Jak často údržba kontroluje, zda je potřeba handoff po změně epochy.
    #[arg(long, env = "MAINTENANCE_INTERVAL_S", default_value_t = 30)]
    pub maintenance_interval_s: u64,

    /// Jak často se uklízí vypršené zprávy.
    #[arg(long, env = "JANITOR_INTERVAL_S", default_value_t = 3600)]
    pub janitor_interval_s: u64,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct Claimed {
    pub cpu_cores: u64,
    pub ram_mb: u64,
    pub storage_gb: u64,
    pub bandwidth_mbps: u64,
}

impl Config {
    pub fn worker_base(&self) -> &str {
        self.worker_url.trim_end_matches('/')
    }

    /// `ws(s)://…/ws` odvozené z `WORKER_URL`.
    pub fn worker_ws_url(&self) -> String {
        let base = self.worker_base();
        let swapped = if let Some(rest) = base.strip_prefix("https://") {
            format!("wss://{rest}")
        } else if let Some(rest) = base.strip_prefix("http://") {
            format!("ws://{rest}")
        } else {
            base.to_owned()
        };
        format!("{swapped}/ws")
    }

    /// Tvrzené specs: přepsání z konfigurace, jinak změřené na hostiteli.
    pub fn claimed(&self, measured: &crate::sysinfo::Measured) -> Claimed {
        Claimed {
            cpu_cores: self.claim_cpu_cores.unwrap_or(measured.cpu_cores).clamp(1, 1024),
            ram_mb: self.claim_ram_mb.unwrap_or(measured.ram_mb).clamp(1, 4_194_304),
            storage_gb: self.claim_storage_gb.unwrap_or(measured.storage_gb).clamp(1, 1_048_576),
            bandwidth_mbps: self.claim_bandwidth_mbps.clamp(1, 400_000),
        }
    }
}
