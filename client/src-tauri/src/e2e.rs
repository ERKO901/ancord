//! Kritérium přijetí D3 (`ARCHITECTURE.md` §13.14): dva klienti přes lokální Worker (`wrangler dev`) a 3 nody
//! (`dev/local.sh 3`) si píší živě, offline příjemce zprávu dostane po připojení, duplicitní doručení se zahodí.
//!
//! Test potřebuje Rust, Node.js a závislosti Workeru (`cd worker && npm install`); bez nich se přeskočí.
//! `ANCORD_SKIP_E2E=1` ho vypne.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use ancord_core::encoding::b64u_decode_array;

use crate::contract::{ConnectionState, DeliveryState, UiEvent};
use crate::core::Core;
use crate::net::{NetConfig, Sink};
use crate::store::KdfParams;

const FAST: KdfParams = KdfParams { m_kib: 8, t: 1 };
const NODES: u16 = 3;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

/// `dev/local.sh 3` ve vlastní skupině procesů; při zániku se zabije celá skupina (wrangler, workerd, nody).
struct LocalNet {
    child: Child,
    run: tempfile::TempDir,
    worker: String,
    forward: u16,
}

impl LocalNet {
    fn start() -> Self {
        let run = tempfile::tempdir().unwrap();
        let (worker_port, forward) = (free_port(), free_port());
        // nody dostanou BASE_PORT+1 … BASE_PORT+N; náhodný základ, aby se test nepotkal s ručně spuštěným local.sh
        let base = 20_000 + u16::try_from(std::process::id() % 20_000).unwrap();
        let log = std::fs::File::create(run.path().join("local.log")).unwrap();
        let child = Command::new("bash")
            .arg(repo().join("dev/local.sh"))
            .arg(NODES.to_string())
            .env("RUN_DIR", run.path().join("net"))
            .env("WPORT", worker_port.to_string())
            .env("FPORT", forward.to_string())
            .env("BASE_PORT", base.to_string())
            .stdin(Stdio::null())
            .stdout(log.try_clone().unwrap())
            .stderr(log)
            .process_group(0)
            .spawn()
            .unwrap();
        Self { child, run, worker: format!("http://127.0.0.1:{worker_port}"), forward }
    }

    /// Čeká, až Worker vydá trasu (3 živé nody, první epocha) a vrátí konfiguraci klienta.
    fn config(&self) -> NetConfig {
        let http = reqwest::Client::new();
        let until = Instant::now() + Duration::from_secs(900);
        let key = loop {
            assert!(Instant::now() < until, "Worker and nodes did not come up, see {}", self.run.path().display());
            let ready = tauri::async_runtime::block_on(async {
                let key: serde_json::Value = http.get(format!("{}/.well-known/ancord-key", self.worker)).send().await.ok()?.json().await.ok()?;
                let key = b64u_decode_array::<32>(key["pubkey"].as_str()?)?;
                ancord_core::net::fetch_route(&http, &self.worker, &[9; 32]).await.ok().filter(|r| r.nodes.len() >= usize::from(NODES))?;
                Some(key)
            });
            if let Some(key) = ready {
                break key;
            }
            std::thread::sleep(Duration::from_secs(2));
        };
        NetConfig {
            worker: self.worker.clone(),
            worker_key: key,
            node_url: Some(format!("ws://127.0.0.1:{}/{{hostname}}/client", self.forward)),
            pow_bits: 10,
        }
    }

    fn dump_logs(&self) {
        let logs = self.run.path().join("net/logs");
        for entry in std::fs::read_dir(&logs).into_iter().flatten().flatten() {
            let text = std::fs::read_to_string(entry.path()).unwrap_or_default();
            let tail: Vec<&str> = text.lines().rev().take(30).collect();
            eprintln!("--- {} ---", entry.path().display());
            for line in tail.iter().rev() {
                eprintln!("{line}");
            }
        }
        eprintln!("--- local.sh ---\n{}", std::fs::read_to_string(self.run.path().join("local.log")).unwrap_or_default());
    }
}

impl Drop for LocalNet {
    fn drop(&mut self) {
        if std::thread::panicking() {
            self.dump_logs();
        }
        let _ = Command::new("kill").args(["-TERM", "--", &format!("-{}", self.child.id())]).status();
        std::thread::sleep(Duration::from_millis(500));
        let _ = Command::new("kill").args(["-KILL", "--", &format!("-{}", self.child.id())]).status();
        let _ = self.child.wait();
    }
}

/// Odchozí spojení vlastního procesu (kritérium D5): vzorkuje `/proc/net/{tcp,tcp6,udp,udp6}` přes sokety z `/proc/self/fd`
/// a sbírá vzdálené adresy. Krátké spojení mezi vzorky může uniknout; klient drží dlouhé WebSocket spojení a opakuje `/route`.
struct Egress {
    seen: Arc<Mutex<std::collections::BTreeSet<String>>>,
    stop: Arc<std::sync::atomic::AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Egress {
    fn start() -> Self {
        let seen = Arc::new(Mutex::new(std::collections::BTreeSet::new()));
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let (s, st) = (seen.clone(), stop.clone());
        let thread = std::thread::spawn(move || {
            while !st.load(std::sync::atomic::Ordering::Relaxed) {
                if let Ok(mut set) = s.lock() {
                    set.extend(remotes());
                }
                std::thread::sleep(Duration::from_millis(20));
            }
        });
        Self { seen, stop, thread: Some(thread) }
    }

    fn finish(mut self) -> Vec<String> {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
        self.seen.lock().map(|s| s.iter().cloned().collect()).unwrap_or_default()
    }
}

/// Vzdálené koncové body (`ip:port`, hex jako v /proc) všech navázaných soketů tohoto procesu.
fn remotes() -> Vec<String> {
    let mut inodes = std::collections::HashSet::new();
    for fd in std::fs::read_dir("/proc/self/fd").into_iter().flatten().flatten() {
        if let Ok(target) = std::fs::read_link(fd.path()) {
            if let Some(i) = target.to_string_lossy().strip_prefix("socket:[").and_then(|t| t.strip_suffix(']')) {
                inodes.insert(i.to_string());
            }
        }
    }
    let mut out = Vec::new();
    for table in ["tcp", "tcp6", "udp", "udp6"] {
        let text = std::fs::read_to_string(format!("/proc/net/{table}")).unwrap_or_default();
        for line in text.lines().skip(1) {
            let cols: Vec<&str> = line.split_whitespace().collect();
            // sl local rem st tx:rx tr:when retrnsmt uid timeout inode
            if let (Some(rem), Some(inode)) = (cols.get(2), cols.get(9)) {
                let unbound = rem.starts_with("00000000000000000000000000000000:0000") || *rem == "00000000:0000";
                if inodes.contains(*inode) && !unbound {
                    out.push(format!("{table} {rem}"));
                }
            }
        }
    }
    out
}

/// `/proc/net/tcp` zapisuje adresu po bajtech v pořadí hostitele (little endian) a port hexadecimálně.
fn is_loopback_port(entry: &str, ports: &[u16]) -> bool {
    let Some((table, rem)) = entry.split_once(' ') else { return false };
    let Some((ip, port)) = rem.split_once(':') else { return false };
    let Ok(port) = u16::from_str_radix(port, 16) else { return false };
    let v4_loopback = ip == "0100007F" || ip == "0000000000000000FFFF00000100007F";
    let v6_loopback = ip == "00000000000000000000000001000000";
    table.starts_with("tcp") && (v4_loopback || v6_loopback) && ports.contains(&port)
}

struct ChanSink(Mutex<Sender<UiEvent>>);

impl Sink for ChanSink {
    fn emit(&self, event: UiEvent) {
        if let Ok(tx) = self.0.lock() {
            let _ = tx.send(event);
        }
    }
}

/// Klient s profilem ve složce `dir` (při prvním spuštění s novou identitou).
fn client(dir: &Path) -> (Core, Receiver<UiEvent>, Arc<dyn Sink>) {
    // zrušená síťová úloha předchozího klienta drží úložiště, dokud ji runtime neuklidí
    let until = Instant::now() + Duration::from_secs(10);
    let core = loop {
        match Core::persistent(dir.to_path_buf(), FAST, false) {
            Ok(core) => break core,
            Err(_) if Instant::now() < until => std::thread::sleep(Duration::from_millis(100)),
            Err(e) => panic!("profile: {e}"),
        }
    };
    if core.address_get().is_err() {
        core.identity_create(None).unwrap();
    }
    let (tx, rx) = channel();
    (core, rx, Arc::new(ChanSink(Mutex::new(tx))))
}

/// Další událost, která splní `want` (ostatní přeskočí); test selže po `secs`.
fn wait(rx: &Receiver<UiEvent>, secs: u64, what: &str, want: impl Fn(&UiEvent) -> bool) -> UiEvent {
    let until = Instant::now() + Duration::from_secs(secs);
    loop {
        let left = until.saturating_duration_since(Instant::now());
        match rx.recv_timeout(left) {
            Ok(e) if want(&e) => return e,
            Ok(_) => {}
            Err(_) => panic!("timeout waiting for {what}"),
        }
    }
}

fn online(rx: &Receiver<UiEvent>, who: &str) {
    wait(rx, 120, &format!("{who} online"), |e| matches!(e, UiEvent::ConnectionState { state: ConnectionState::Online }));
}

fn received(rx: &Receiver<UiEvent>, what: &str) -> (String, String) {
    match wait(rx, 60, what, |e| matches!(e, UiEvent::MessageReceived { .. })) {
        UiEvent::MessageReceived { conversation_id, message } => {
            assert!(!message.mine);
            (conversation_id, message.text)
        }
        _ => unreachable!(),
    }
}

fn state_of(rx: &Receiver<UiEvent>, id: &str, state: DeliveryState) {
    wait(rx, 60, &format!("{state:?} for {id}"), |e| matches!(e, UiEvent::DeliveryState { message_id, state: s, .. } if message_id == id && *s == state));
}

fn texts(core: &Core, conv: &str) -> Vec<(bool, String)> {
    core.conversation_open(conv, None, 100).unwrap().messages.into_iter().map(|m| (m.mine, m.text)).collect()
}

#[test]
fn two_clients_chat_live_offline_and_without_duplicates() {
    if std::env::var_os("ANCORD_SKIP_E2E").is_some() || !repo().join("worker/node_modules").is_dir() {
        eprintln!("e2e skipped (ANCORD_SKIP_E2E or missing worker/node_modules)");
        return;
    }
    let net = LocalNet::start();
    let config = net.config();
    let egress = Egress::start();
    let (dir_a, dir_b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());

    // Bob je online; Alice zatím bez sítě, takže její první zpráva čeká ve frontě
    let (bob, rx_b, sink_b) = client(dir_b.path());
    bob.go_online(config.clone(), sink_b.clone()).unwrap();
    online(&rx_b, "Bob");
    let (alice, rx_a, sink_a) = client(dir_a.path());
    let to_bob = alice.contact_add(&bob.address_get().unwrap(), Some("Bob".into())).unwrap();
    let first = alice.message_send(&to_bob.id, "ahoj z fronty").unwrap();
    assert_eq!(first.state, DeliveryState::Queued);

    // Alice se připojí: založí konverzaci podle adresy, odešle frontu a Bob ji živě dostane
    alice.go_online(config.clone(), sink_a.clone()).unwrap();
    online(&rx_a, "Alice");
    let request = wait(&rx_b, 60, "contact request", |e| matches!(e, UiEvent::ContactRequest { .. }));
    let UiEvent::ContactRequest { contact: alice_at_bob } = request else { unreachable!() };
    assert_eq!(alice_at_bob.address, alice.address_get().unwrap());
    assert_eq!(received(&rx_b, "first message"), (alice_at_bob.id.clone(), "ahoj z fronty".to_string()));
    state_of(&rx_a, &first.id, DeliveryState::Delivered);

    // odpověď a psaní živě oběma směry
    let reply = bob.message_send(&alice_at_bob.id, "ahoj Alice").unwrap();
    assert_eq!(received(&rx_a, "reply"), (to_bob.id.clone(), "ahoj Alice".to_string()));
    state_of(&rx_b, &reply.id, DeliveryState::Delivered);
    alice.typing_set(&to_bob.id, true);
    wait(&rx_b, 30, "typing", |e| matches!(e, UiEvent::Typing { conversation_id, on: true } if *conversation_id == alice_at_bob.id));
    assert_eq!(bob.contact_safety_number(&alice_at_bob.id).unwrap(), alice.contact_safety_number(&to_bob.id).unwrap());

    // offline příjemce: Bob skončí, zpráva čeká na anchorech (pending) a dorazí po jeho připojení
    drop(bob);
    std::thread::sleep(Duration::from_secs(2));
    let late = alice.message_send(&to_bob.id, "když jsi byl pryč").unwrap();
    state_of(&rx_a, &late.id, DeliveryState::Pending);
    let (bob, rx_b, sink_b) = client(dir_b.path());
    bob.go_online(config.clone(), sink_b).unwrap();
    assert_eq!(received(&rx_b, "message after reconnect"), (alice_at_bob.id.clone(), "když jsi byl pryč".to_string()));
    state_of(&rx_a, &late.id, DeliveryState::Delivered);

    // bez duplicit: každá zpráva právě jednou, ani po dalším restartu (znovu `attach` a `fetch`)
    let want = vec![(false, "ahoj z fronty".to_string()), (true, "ahoj Alice".to_string()), (false, "když jsi byl pryč".to_string())];
    assert_eq!(texts(&bob, &alice_at_bob.id), want);
    drop(bob);
    let (bob, rx_b2, sink_b2) = client(dir_b.path());
    bob.go_online(config.clone(), sink_b2).unwrap();
    online(&rx_b2, "Bob again");
    let quiet = Instant::now() + Duration::from_secs(5);
    while let Ok(e) = rx_b2.recv_timeout(quiet.saturating_duration_since(Instant::now())) {
        assert!(!matches!(e, UiEvent::MessageReceived { .. }), "duplicate delivery: {e:?}");
    }
    assert_eq!(texts(&bob, &alice_at_bob.id), want);
    assert_eq!(texts(&alice, &to_bob.id).len(), 3);
    let list = bob.conversation_list().unwrap();
    assert_eq!((list.len(), list[0].last_text.as_str()), (1, "když jsi byl pryč"));

    // D6: `sync_once` (Android po wake): Bob je offline, vyzvedne zprávu jednorázově, podruhé už nic
    drop(bob);
    std::thread::sleep(Duration::from_secs(2));
    alice.message_send(&to_bob.id, "pro sync_once").unwrap();
    let (bob, rx_b3, sink_b3) = client(dir_b.path());
    assert_eq!(bob.sync_once(&config, sink_b3.clone(), crate::net::SYNC_LIMIT).unwrap(), 1);
    assert_eq!(received(&rx_b3, "message from sync_once"), (alice_at_bob.id.clone(), "pro sync_once".to_string()));
    assert_eq!(bob.sync_once(&config, sink_b3, crate::net::SYNC_LIMIT).unwrap(), 0, "ack až po uložení, žádná duplicita");
    assert_eq!(texts(&bob, &alice_at_bob.id).len(), 4);

    // D5: za celou dobu klienti mluvili jen s Workerem a s přesměrováním k nodům (`dev/local.sh`), nic jiného
    let allowed = [net.worker.rsplit(':').next().and_then(|p| p.parse::<u16>().ok()).unwrap(), net.forward];
    let seen = egress.finish();
    assert!(!seen.is_empty(), "sampler saw no connections");
    let stray: Vec<&String> = seen.iter().filter(|e| !is_loopback_port(e, &allowed)).collect();
    assert!(stray.is_empty(), "unexpected outbound connections: {stray:?}");
}
