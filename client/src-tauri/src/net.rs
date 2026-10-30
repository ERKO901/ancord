//! Online klient (milník D3, §13.7). Jedna úloha drží `ancord_core::online::Online` a jedno spojení s gateway:
//! `/route`, admission s ověřením ticketu připnutým klíčem Workeru, `attach` (profil, stráž, odběry, co čekalo
//! offline), potom zprávy živě. Po pádu spojení se připojí znovu s novým efemerním klíčem a novou trasou.
//!
//! Úloha je jediný vlastník stavu ratchetu za běhu; `Core` jí posílá jen pokyny (`Cmd`). Zrušit ji jde kdykoli
//! (`Link` při zániku úlohu ukončí): stav se ukládá před `ack` i před odesláním, takže přerušení nic neztratí.

use std::sync::Arc;
use std::time::{Duration, Instant};

use ancord_core::encoding::b64u_decode_array;
use ancord_core::net::{connect_any, default_url, fetch_route, message_status, send_typing, CliError, NodeSocket};
use ancord_core::online::{Online, OnlineEvent};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use tokio::sync::mpsc;

use crate::contract::{ConnectionIssue, ConnectionState, DeliveryState, UiEvent};
use crate::core::{session_conv, Shared, StorePersist};

/// Ping při nečinnosti (idle timeout tunelu je 125 s, §9).
const PING_EVERY: Duration = Duration::from_secs(25);
/// Jak často se ptát anchorů na stav odeslaných zpráv, dokud nějaká čeká na doručení.
const STATUS_EVERY: Duration = Duration::from_secs(2);
/// Nejdelší pauza mezi pokusy o připojení.
const MAX_BACKOFF_S: u64 = 60;

/// Kam se klient připojuje. Kořeny důvěry (§13.3).
#[derive(Clone, Debug)]
pub struct NetConfig {
    pub worker: String,
    /// Veřejný klíč Workeru: bez něj by ticket i node mohl podvrhnout kdokoli na cestě, včetně Cloudflare.
    pub worker_key: [u8; 32],
    /// Šablona adresy nodu, `{hostname}` se nahradí (výchozí `wss://{hostname}/client`).
    pub node_url: Option<String>,
    /// Obtížnost hashcash profilu; musí odpovídat `PROFILE_POW_BITS` nodů.
    pub pow_bits: u32,
}

impl NetConfig {
    /// Hodnoty zapečené při buildu (`ANCORD_WORKER_URL`, `ANCORD_WORKER_PUBKEY`, volitelně `ANCORD_NODE_URL`
    /// a `ANCORD_POW_BITS`). Debug build je smí přepsat proměnnými prostředí stejného jména, release ne.
    /// Bez URL a klíče Workeru zůstane klient offline (`None`).
    pub fn from_build() -> Option<Self> {
        let get = |name: &str, baked: Option<&'static str>| {
            let runtime = if cfg!(debug_assertions) { std::env::var(name).ok() } else { None };
            runtime.or_else(|| baked.map(str::to_owned)).filter(|v| !v.is_empty())
        };
        let worker = get("ANCORD_WORKER_URL", option_env!("ANCORD_WORKER_URL"))?;
        let worker_key = get("ANCORD_WORKER_PUBKEY", option_env!("ANCORD_WORKER_PUBKEY")).and_then(|k| b64u_decode_array::<32>(&k))?;
        let node_url = get("ANCORD_NODE_URL", option_env!("ANCORD_NODE_URL"));
        let pow_bits = get("ANCORD_POW_BITS", option_env!("ANCORD_POW_BITS")).and_then(|b| b.parse().ok()).unwrap_or(10);
        Some(Self { worker, worker_key, node_url, pow_bits })
    }

    fn url_for(&self, hostname: &str) -> String {
        self.node_url.as_ref().map_or_else(|| default_url(hostname), |t| t.replace("{hostname}", hostname))
    }
}

/// Kam jdou události pro UI (v aplikaci `AppHandle::emit`, v testech kanál).
pub trait Sink: Send + Sync {
    fn emit(&self, event: UiEvent);
}

/// Pokyny z příkazů UI.
#[derive(Debug)]
pub enum Cmd {
    /// Přibyl kontakt nebo zpráva k odeslání.
    Flush,
    Typing { id: String, on: bool },
}

/// Běžící síťová úloha. Zánik `Link` úlohu ukončí.
pub struct Link {
    tx: mpsc::UnboundedSender<Cmd>,
    task: tauri::async_runtime::JoinHandle<()>,
}

impl Link {
    pub fn send(&self, cmd: Cmd) {
        let _ = self.tx.send(cmd);
    }

    pub fn finished(&self) -> bool {
        self.tx.is_closed()
    }
}

impl Drop for Link {
    fn drop(&mut self) {
        self.task.abort();
    }
}

pub(crate) fn start(shared: Shared, config: Arc<NetConfig>, sink: Arc<dyn Sink>) -> Link {
    let (tx, rx) = mpsc::unbounded_channel();
    let task = tauri::async_runtime::spawn(run(shared, config, sink, rx));
    Link { tx, task }
}

fn set_state(shared: &Shared, sink: &dyn Sink, state: ConnectionState) {
    if shared.set_conn(state) {
        sink.emit(UiEvent::ConnectionState { state });
    }
}

async fn run(shared: Shared, config: Arc<NetConfig>, sink: Arc<dyn Sink>, mut rx: mpsc::UnboundedReceiver<Cmd>) {
    let Ok((keys, sessions)) = shared.load_online() else { return };
    let mut online = Online::new(keys, sessions, config.pow_bits);
    let mut task = Task { shared: shared.clone(), sink: sink.clone(), persist: StorePersist::new(shared.clone()) };
    let http = reqwest::Client::new();
    let mut backoff = 1u64;
    loop {
        set_state(&shared, &*sink, ConnectionState::Connecting);
        // každé připojení má nový efemerní klíč a novou trasu (failover i na jiné nody)
        let ek = SigningKey::generate(&mut OsRng);
        let connected = match fetch_route(&http, &config.worker, &ek.verifying_key().to_bytes()).await {
            Ok(route) => connect_any(&route, &ek, &config.worker_key, |h| config.url_for(h)).await.map(|(_, ws)| ws),
            Err(e) => Err(e),
        };
        if let Err(e) = &connected {
            #[cfg(debug_assertions)]
            eprintln!("ancord net: connect failed: {e}");
            sink.emit(UiEvent::ConnectionIssue { issue: Some(classify(e)) });
        }
        if let Ok(mut ws) = connected {
            let result = task.serve(&mut online, &mut ws, &mut rx).await;
            ws.close().await;
            if let Err(e) = &result {
                #[cfg(debug_assertions)]
                eprintln!("ancord net: connection ended: {e}");
                sink.emit(UiEvent::ConnectionIssue { issue: Some(classify(e)) });
            }
            if result.is_ok() {
                // kanál pokynů zanikl: `Core` síť vypnul
                set_state(&shared, &*sink, ConnectionState::Offline);
                return;
            }
            backoff = 1;
        }
        set_state(&shared, &*sink, ConnectionState::Offline);
        let pause = tokio::time::sleep(Duration::from_secs(backoff));
        tokio::pin!(pause);
        loop {
            tokio::select! {
                () = &mut pause => break,
                cmd = rx.recv() => match cmd {
                    None => return,
                    // uživatel něco odesílá: zkusit to hned
                    Some(Cmd::Flush) => break,
                    Some(Cmd::Typing { .. }) => {}
                },
            }
        }
        backoff = (backoff * 2).min(MAX_BACKOFF_S);
    }
}

/// Výchozí tvrdý limit `sync_once` (§13.4).
#[allow(dead_code, reason = "viz Core::sync_once")]
pub const SYNC_LIMIT: Duration = Duration::from_secs(20);

/// Jednorázové vyzvednutí po wake (§13.4): `/route` → připojení → `attach` (profil, odběry, `fetch`) → odpojení.
/// Zpráva se uloží (včetně stavu ratchetu) dřív, než `Online` pošle `ack`; po vypršení `limit` se zahodí jen
/// rozpracované čekání na síť, nic potvrzeného se neztratí. Vrací počet přijatých zpráv.
/// Smí běžet jen tam, kde neběží `Link` (jeden vlastník stavu ratchetu).
#[allow(dead_code, reason = "viz Core::sync_once")]
pub(crate) async fn sync_once(shared: Shared, config: &NetConfig, sink: Arc<dyn Sink>, limit: Duration) -> Result<usize, CliError> {
    let (keys, sessions) = shared.load_online().map_err(CliError::Session)?;
    let mut online = Online::new(keys, sessions, config.pow_bits);
    let mut persist = StorePersist::new(shared);
    let fetch = async {
        let http = reqwest::Client::new();
        let ek = SigningKey::generate(&mut OsRng);
        let route = fetch_route(&http, &config.worker, &ek.verifying_key().to_bytes()).await?;
        let (_, ws) = connect_any(&route, &ek, &config.worker_key, |h| config.url_for(h)).await?;
        Ok::<_, CliError>(ws)
    };
    let mut ws = tokio::time::timeout(limit, fetch).await.map_err(|_| CliError::Connection("sync timed out".into()))??;
    // `attach` se nesmí přerušit uprostřed (viz `Online::handle`), proto limit hlídá jen spojení a samotné čekání
    let attached = online.attach(&mut ws, &mut persist).await;
    ws.close().await;
    attached?;
    let events = persist.take_events();
    let received = events.iter().filter(|e| matches!(e, UiEvent::MessageReceived { .. })).count();
    for e in events {
        sink.emit(e);
    }
    Ok(received)
}

/// Hrubý důvod selhání pro UI (spojení se rozlišuje podle varianty chyby a textu admission).
fn classify(e: &CliError) -> ConnectionIssue {
    match e {
        CliError::Route(m) if m.starts_with(|c: char| c.is_ascii_digit()) || m.starts_with("unexpected response") => ConnectionIssue::WorkerError,
        CliError::Route(_) => ConnectionIssue::WorkerUnreachable,
        CliError::Admission { reason, .. } if reason.contains("protocol mismatch") => ConnectionIssue::ProtocolMismatch,
        CliError::AllFailed(m) if m.contains("protocol mismatch") => ConnectionIssue::ProtocolMismatch,
        CliError::Admission { reason, .. } if reason.contains("untrusted route") => ConnectionIssue::RouteUntrusted,
        CliError::AllFailed(m) if m.contains("untrusted route") => ConnectionIssue::RouteUntrusted,
        CliError::Node { .. } => ConnectionIssue::NodeRejected,
        _ => ConnectionIssue::NodeUnreachable,
    }
}

struct Task {
    shared: Shared,
    sink: Arc<dyn Sink>,
    persist: StorePersist,
}

impl Task {
    /// Chyba spojení ukončí spojení (a přijde nové), ostatní (node odmítl, profil nenalezen) jen přeskočí jednu akci.
    /// Odmítnutí `bad_request` se hlásí uživateli: nasazený node nejspíš nezná zprávu této verze klienta.
    fn soft(&self, e: CliError) -> Result<(), CliError> {
        match e {
            CliError::Connection(_) => Err(e),
            CliError::Node { ref code, .. } => {
                if code == "bad_request" {
                    self.sink.emit(UiEvent::ConnectionIssue { issue: Some(ConnectionIssue::NodeRejected) });
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Jedno spojení. `Ok` = kanál pokynů zanikl, `Err` = spojení spadlo.
    async fn serve(&mut self, online: &mut Online, ws: &mut NodeSocket, rx: &mut mpsc::UnboundedReceiver<Cmd>) -> Result<(), CliError> {
        let (events, _skipped) = online.attach(ws, &mut self.persist).await?;
        self.sink.emit(UiEvent::ConnectionIssue { issue: None });
        // online = profil zveřejněný a odběry přihlášené: protistrana nás teď najde a zprávy chodí živě
        set_state(&self.shared, &*self.sink, ConnectionState::Online);
        self.settle(online, events);
        self.flush(online, ws).await?;
        let mut last_ping = Instant::now();
        let mut last_status = Instant::now();
        loop {
            let awaiting = self.shared.has_awaiting();
            let wait = if awaiting { STATUS_EVERY } else { PING_EVERY };
            // čekání na rámec nebo pokyn lze přerušit; zpracování (`handle`, `flush`) běží vždy celé
            tokio::select! {
                frame = ws.recv_json_timeout(wait) => {
                    if let Some(frame) = frame? {
                        let events = online.handle(ws, &mut self.persist, &frame).await?;
                        self.settle(online, events);
                    }
                }
                cmd = rx.recv() => match cmd {
                    None => return Ok(()),
                    Some(Cmd::Flush) => self.flush(online, ws).await?,
                    Some(Cmd::Typing { id, on }) => {
                        if let Some(s) = online.conversations.iter().find(|s| session_conv(s).as_deref() == Some(id.as_str())) {
                            send_typing(ws, s, on).await.or_else(|e| self.soft(e))?;
                        }
                    }
                },
            }
            if awaiting && last_status.elapsed() >= STATUS_EVERY {
                self.statuses(online, ws).await?;
                last_status = Instant::now();
            }
            if last_ping.elapsed() >= PING_EVERY {
                ws.send_json(&serde_json::json!({"t": "ping"})).await?;
                last_ping = Instant::now();
                // kontakt, jehož profil zatím nešel najít, a zprávy, které node odmítl: zkusit znovu
                self.flush(online, ws).await?;
            }
            online.keepalives(ws, &mut self.persist).await?;
            self.settle(online, Vec::new());
        }
    }

    /// Po operaci `Online`: předá události UI a vyprázdní historii konverzací (zprávy už jsou v úložišti).
    fn settle(&mut self, online: &mut Online, events: Vec<OnlineEvent>) {
        let mut ui = self.persist.take_events();
        for e in events {
            if let OnlineEvent::Typing { index, on } = e {
                if let Some(conv) = online.conversations.get(index).and_then(session_conv) {
                    ui.push(UiEvent::Typing { conversation_id: conv, on });
                }
            }
        }
        for s in &mut online.conversations {
            s.history.clear();
        }
        for e in ui {
            self.sink.emit(e);
        }
    }

    /// Založí konverzace s novými kontakty a odešle zprávy z fronty.
    async fn flush(&mut self, online: &mut Online, ws: &mut NodeSocket) -> Result<(), CliError> {
        let have: Vec<String> = online.conversations.iter().filter_map(session_conv).collect();
        for pk in self.shared.contacts_without(&have) {
            match online.start(ws, &mut self.persist, &pk).await {
                Ok(event) => self.settle(online, vec![event]),
                // protistrana ještě nezveřejnila profil: zkusí se při dalším připojení nebo odeslání
                Err(e) => self.soft(e)?,
            }
        }
        for (conv, text) in self.shared.intros() {
            let Some(index) = online.conversations.iter().position(|s| session_conv(s).as_deref() == Some(conv.as_str())) else { continue };
            let sent = online.send(ws, &mut self.persist, index, &text).await;
            self.settle(online, Vec::new());
            match sent {
                Ok(_) => self.shared.intro_done(&conv)?,
                Err(e) => self.soft(e)?,
            }
        }
        for (conv, key, text) in self.shared.queued() {
            let Some(index) = online.conversations.iter().position(|s| session_conv(s).as_deref() == Some(conv.as_str())) else { continue };
            let sent = online.send(ws, &mut self.persist, index, &text).await;
            self.settle(online, Vec::new());
            match sent {
                Ok(id) => {
                    if let Some(event) = self.shared.mark_sent(&conv, &key, &id)? {
                        self.sink.emit(event);
                    }
                }
                Err(e) => self.soft(e)?,
            }
        }
        Ok(())
    }

    /// Zeptá se anchorů na stav odeslaných zpráv (§6.4) a konečné stavy předá UI.
    async fn statuses(&mut self, online: &Online, ws: &mut NodeSocket) -> Result<(), CliError> {
        for (conv, items) in self.shared.awaiting() {
            let Some(session) = online.conversations.iter().find(|s| session_conv(s).as_deref() == Some(conv.as_str())) else { continue };
            let ids: Vec<[u8; 16]> = items.iter().map(|(_, id)| *id).collect();
            let states = match message_status(ws, session, &ids).await {
                Ok(s) => s,
                Err(e) => {
                    self.soft(e)?;
                    continue;
                }
            };
            let done: Vec<(String, DeliveryState)> = items
                .iter()
                .zip(states)
                .filter_map(|((key, _), state)| match state.as_str() {
                    "delivered" => Some((key.clone(), DeliveryState::Delivered)),
                    "unknown" => Some((key.clone(), DeliveryState::Unknown)),
                    _ => None,
                })
                .collect();
            if !done.is_empty() {
                for event in self.shared.finish(&conv, &done)? {
                    self.sink.emit(event);
                }
            }
        }
        Ok(())
    }
}
