use std::process::ExitCode;

use std::path::{Path, PathBuf};
use std::time::Duration;

use ancord_cli::online::{Online, OnlineEvent, Persist};
use ancord_cli::{
    ack_inbox, address, connect_any, create_user, default_url, fetch_route, load_user, lock_file, lookup_profile, message_status, next_event, parse_address,
    process_inbox, publish_profile, receive, register_push, safety, save_user, send_typing, start_conversation, subscribe, transmit, write_private, CliError,
    Event, NodeSocket, Role, Route, RouteNode, Session,
};
use ancord_node::user::UserKeys;
use ancord_node::encoding::{b64u_decode_array, b64u_encode};
use clap::{Parser, Subcommand, ValueEnum};
use rand::RngCore;
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncBufReadExt;

#[derive(Parser)]
#[command(name = "ancord", about = "ancord CLI klient")]
struct Cli {
    /// Adresa Workeru.
    #[arg(long, env = "WORKER_URL")]
    worker: String,
    /// Veřejný klíč Workeru (b64u, 32 B), připnutý mimo síť. Bez něj klient nepřijme žádný node: tickety i nody by
    /// jinak mohl podvrhnout kdokoli na cestě, včetně Cloudflare. Zjistí se při nasazení Workeru z `/.well-known/ancord-key`.
    #[arg(long, env = "WORKER_PUBKEY", global = true)]
    worker_pubkey: Option<String>,
    /// Šablona adresy nodu; `{hostname}` se nahradí. Výchozí `wss://{hostname}/client`.
    #[arg(long, global = true)]
    via: Option<String>,
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Vypíše nody z `/route` (s novým efemerním klíčem).
    Route,
    /// Připojí se na první node, který klienta přijme, a pošle ping.
    Connect,
    /// Založí soubor konverzace. Tajemství se předá protistraně mimo síť; ta ho zadá s opačnou rolí.
    Init {
        file: PathBuf,
        #[arg(long, value_enum)]
        role: RoleArg,
        /// Sdílené tajemství (b64u, 32 B). Bez něj se vygeneruje a vypíše.
        #[arg(long)]
        secret: Option<String>,
    },
    /// Vytvoří klíče uživatele (adresu a klíč pro dohodu tajemství) a uloží je do souboru s právy 0600.
    Keygen { file: PathBuf },
    /// Vypíše adresu uživatele (`ancord:<veřejný klíč>`), kterou předáte protistraně textem nebo QR kódem.
    Whoami {
        file: PathBuf,
        /// Vypíše adresu i jako QR kód (k naskenování druhým zařízením).
        #[arg(long)]
        qr: bool,
    },
    /// Zaregistruje push token u Workeru pro schránku a všechny konverzace ve složce a uloží ho (`<dir>/push.json`),
    /// aby `online` registroval i nové konverzace. Offline uživatele pak anchory budí pushem.
    PushRegister {
        user: PathBuf,
        dir: PathBuf,
        #[arg(long, value_parser = ["fcm", "apns"])]
        platform: String,
        #[arg(long)]
        token: String,
    },
    /// Připojí se a zůstane online: odebírá schránku i všechny konverzace ve složce, nové kontakty přijímá sám a
    /// zprávy vypisuje, jak přicházejí. Ze vstupu čte `<jméno>: <text>` (odeslat), `/start <adresa> [jméno]`
    /// (nová konverzace) a `/list`. Po výpadku nodu se připojí znovu. Konec Ctrl-C.
    Online {
        user: PathBuf,
        dir: PathBuf,
        /// Obtížnost hashcash profilu; musí odpovídat PROFILE_POW_BITS nodů.
        #[arg(long, default_value_t = 10)]
        pow_bits: u32,
    },
    /// Zveřejní profil uživatele na anchorech, aby vás protistrana našla podle adresy.
    Publish {
        file: PathBuf,
        /// Obtížnost hashcash profilu; musí odpovídat PROFILE_POW_BITS nodů.
        #[arg(long, default_value_t = 10)]
        pow_bits: u32,
    },
    /// Najde a ověří profil podle adresy.
    Lookup { address: String },
    /// Založí konverzaci s uživatelem podle jeho adresy: najde profil, ověří ho, provede X3DH a pošle protistraně
    /// šifrovanou obálku. Nikdo si nemusí nic předávat předem. Protistrana ji zpracuje příkazem `inbox`.
    Start { user: PathBuf, address: String, conversation: PathBuf },
    /// Vyzvedne úvodní obálky ze schránky a založí konverzace do složky (`<adresa>.json`). Použité jednorázové
    /// prekeys se nenávratně smažou, proto se klíče ukládají před potvrzením obálek.
    Inbox { user: PathBuf, dir: PathBuf },
    /// Bezpečnostní číslo s protistranou: porovnejte ho mimo síť (osobně, telefonem). Liší-li se, někdo podvrhl klíč.
    Safety { user: PathBuf, address: String },
    /// Řekne protistraně, že píšete (`--off`: přestali jste).
    Typing {
        file: PathBuf,
        #[arg(long)]
        off: bool,
    },
    /// Zjistí, zda protistrana odeslané zprávy přijala (id vypíše `send`).
    Status { file: PathBuf, ids: Vec<String> },
    /// Vypíše lokální historii konverzace (server historii nedrží).
    History { file: PathBuf },
    /// Zašifruje a odešle zprávu.
    Send { file: PathBuf, text: String },
    /// Stáhne, dešifruje a potvrdí příchozí zprávy.
    Recv { file: PathBuf },
    /// Odebere zprávy, které čekaly, a pak je vypisuje, jak přicházejí (odběr mailboxu u gateway). Konec Ctrl-C.
    Watch { file: PathBuf },
}

#[derive(Clone, Copy, ValueEnum)]
enum RoleArg {
    Initiator,
    Responder,
}

async fn open(cli: &Cli, ek: &SigningKey, route: &Route) -> Result<(RouteNode, NodeSocket), Box<dyn std::error::Error>> {
    let url_for = |host: &str| cli.via.as_ref().map_or_else(|| default_url(host), |t| t.replace("{hostname}", host));
    let worker_key: [u8; 32] = cli
        .worker_pubkey
        .as_deref()
        .and_then(b64u_decode_array)
        .ok_or("WORKER_PUBKEY (32 bytes, base64url) is required to authenticate nodes")?;
    Ok(connect_any(route, ek, &worker_key, url_for).await?)
}

fn load_session(file: &Path) -> Result<Session, Box<dyn std::error::Error>> {
    Ok(Session::load(file)?)
}

fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// Push token uložený u konverzací (`push.json`), aby se dal registrovat i pro nové konverzace.
#[derive(Serialize, Deserialize)]
struct PushConfig {
    platform: String,
    token: String,
}

const PUSH_FILE: &str = "push.json";

fn load_push(dir: &Path) -> Result<Option<PushConfig>, Box<dyn std::error::Error>> {
    match std::fs::read_to_string(dir.join(PUSH_FILE)) {
        Ok(text) => Ok(Some(serde_json::from_str(&text)?)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Soubory konverzací ve složce (`*.json` kromě `push.json`), seřazené podle jména.
fn conversation_files(dir: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|e| e == "json") && path.file_name().is_some_and(|n| n != PUSH_FILE) {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn label_of(path: &Path) -> String {
    path.file_stem().map_or_else(String::new, |s| s.to_string_lossy().into_owned())
}

/// Konverzace online klienta: soubory ve složce, každý zamčený po celou dobu běhu.
struct DirStore {
    user: PathBuf,
    dir: PathBuf,
    paths: Vec<PathBuf>,
    locks: Vec<std::fs::File>,
    /// Jméno pro další založenou konverzaci (`/start … jméno`).
    next_name: Option<String>,
}

impl DirStore {
    fn label(&self, index: usize) -> String {
        self.paths.get(index).map_or_else(String::new, |p| label_of(p))
    }

    fn index_of(&self, label: &str) -> Option<usize> {
        self.paths.iter().position(|p| label_of(p) == label)
    }

    /// Volné jméno souboru: zadané, jinak začátek adresy protistrany.
    fn free_path(&self, session: &Session) -> PathBuf {
        let peer = session.peer.clone().unwrap_or_else(|| "conversation".into());
        let wanted = self.next_name.clone().unwrap_or_else(|| peer.chars().take(8).collect());
        let mut path = self.dir.join(format!("{wanted}.json"));
        let mut n = 2;
        while path.exists() {
            path = self.dir.join(format!("{wanted}-{n}.json"));
            n += 1;
        }
        path
    }
}

impl Persist for DirStore {
    fn save_keys(&mut self, keys: &UserKeys) -> Result<(), CliError> {
        save_user(keys, &self.user)
    }

    fn save_conversation(&mut self, index: usize, session: &Session) -> Result<(), CliError> {
        let path = self.paths.get(index).ok_or_else(|| CliError::Session("no such conversation".into()))?;
        session.save(path)
    }

    fn add_conversation(&mut self, _index: usize, session: &Session) -> Result<(), CliError> {
        let path = self.free_path(session);
        self.next_name = None;
        self.locks.push(lock_file(&path)?);
        write_private(&path, "", true)?;
        session.save(&path)?;
        self.paths.push(path);
        Ok(())
    }
}

fn valid_name(name: &str) -> bool {
    (1..=32).contains(&name.len()) && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

/// Registruje push token pro schránku (`inbox`) a dané konverzace (spec §15.11).
async fn register_all(
    cli: &Cli,
    http: &reqwest::Client,
    push: &PushConfig,
    keys: &UserKeys,
    sessions: &[&Session],
    inbox: bool,
) -> Result<(), CliError> {
    let ts = now();
    if inbox {
        register_push(http, &cli.worker, keys.push_key(), &push.platform, &push.token, ts).await?;
    }
    for s in sessions {
        register_push(http, &cli.worker, &s.inbox_push_key(), &push.platform, &push.token, ts).await?;
    }
    Ok(())
}

struct OnlineCtx<'a> {
    cli: &'a Cli,
    http: reqwest::Client,
    push: Option<PushConfig>,
    store: DirStore,
}

impl OnlineCtx<'_> {
    async fn report(&mut self, online: &Online, events: Vec<OnlineEvent>) {
        for event in events {
            match event {
                OnlineEvent::Message { index, text } => println!("{}: {}", self.store.label(index), String::from_utf8_lossy(&text)),
                OnlineEvent::Typing { index, on } => eprintln!("({} {})", self.store.label(index), if on { "is typing…" } else { "stopped typing" }),
                OnlineEvent::Postponed(n) => eprintln!("{n} handshake(s) postponed (sender profile not found yet)"),
                OnlineEvent::Contact { index, watched } => {
                    let Some(session) = online.conversations.get(index) else { continue };
                    let peer = session.peer.as_deref().and_then(b64u_decode_array::<32>).unwrap_or_default();
                    eprintln!(
                        "+ conversation `{}` with {}\n  safety number {}",
                        self.store.label(index),
                        ancord_node::user::address_uri(&peer),
                        safety(&online.keys, &peer)
                    );
                    if !watched {
                        eprintln!("  not watched live: one connection carries at most 31 conversations");
                    }
                    if let Some(push) = &self.push {
                        if let Err(e) = register_all(self.cli, &self.http, push, &online.keys, &[session], false).await {
                            eprintln!("  push registration failed: {e}");
                        }
                    }
                }
            }
        }
    }

    /// Řádek ze vstupu: `<jméno>: <text>`, `/start <adresa> [jméno]`, `/list`.
    async fn command(&mut self, online: &mut Online, ws: &mut NodeSocket, line: &str) -> Result<(), CliError> {
        let line = line.trim();
        if line.is_empty() {
            return Ok(());
        }
        if line == "/list" {
            for (i, s) in online.conversations.iter().enumerate() {
                println!("{}  {}", self.store.label(i), s.peer.as_deref().unwrap_or("-"));
            }
            return Ok(());
        }
        if let Some(rest) = line.strip_prefix("/start ") {
            let mut parts = rest.split_whitespace();
            let peer = parse_address(parts.next().unwrap_or(""))?;
            if let Some(name) = parts.next() {
                if !valid_name(name) || self.store.index_of(name).is_some() {
                    return Err(CliError::Session(format!("`{name}` is taken or not a valid name (letters, digits, - and _)")));
                }
                self.store.next_name = Some(name.to_owned());
            }
            let event = online.start(ws, &mut self.store, &peer).await;
            self.store.next_name = None;
            self.report(online, vec![event?]).await;
            return Ok(());
        }
        let Some((name, text)) = line.split_once(':') else {
            return Err(CliError::Session("expected `<name>: <text>`, `/start <address> [name]` or `/list`".into()));
        };
        let index = self.store.index_of(name.trim()).ok_or_else(|| CliError::Session(format!("no conversation `{}` (see /list)", name.trim())))?;
        online.send(ws, &mut self.store, index, text.trim_start().as_bytes()).await.map(|_| ())
    }
}

enum Step {
    Frame(Result<Option<serde_json::Value>, CliError>),
    Line(std::io::Result<Option<String>>),
    Quit,
}

/// Jedno spojení online klienta. `Ok(true)` = uživatel skončil, `Err` = spojení spadlo a připojí se znovu.
async fn online_session<R: tokio::io::AsyncBufRead + Unpin>(
    ctx: &mut OnlineCtx<'_>,
    online: &mut Online,
    ws: &mut NodeSocket,
    stdin: &mut Option<tokio::io::Lines<R>>,
) -> Result<bool, CliError> {
    let (events, skipped) = online.attach(ws, &mut ctx.store).await?;
    for i in skipped {
        eprintln!("`{}` not watched live: one connection carries at most 31 conversations", ctx.store.label(i));
    }
    ctx.report(online, events).await;
    loop {
        // čekání na rámec nebo řádek lze přerušit; zpracování (`handle`, `command`) běží vždy celé
        let step = tokio::select! {
            f = ws.recv_json_timeout(Duration::from_secs(25)) => Step::Frame(f),
            l = async { match stdin.as_mut() { Some(s) => s.next_line().await, None => std::future::pending().await } } => Step::Line(l),
            _ = tokio::signal::ctrl_c() => Step::Quit,
        };
        match step {
            Step::Quit => return Ok(true),
            Step::Frame(Ok(None)) => ws.send_json(&serde_json::json!({"t": "ping"})).await?,
            Step::Frame(Ok(Some(frame))) => {
                let events = online.handle(ws, &mut ctx.store, &frame).await?;
                ctx.report(online, events).await;
            }
            Step::Frame(Err(e)) => return Err(e),
            Step::Line(Ok(Some(line))) => match ctx.command(online, ws, &line).await {
                Err(e @ CliError::Connection(_)) => return Err(e),
                Err(e) => eprintln!("error: {e}"),
                Ok(()) => {}
            },
            // konec vstupu (např. běh na pozadí): dál jen přijímat
            Step::Line(_) => *stdin = None,
        }
    }
}

async fn run_online(cli: &Cli, user: &Path, dir: &Path, pow_bits: u32) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(dir)?;
    let user_lock = lock_file(user)?;
    let keys = load_user(user)?;
    let paths = conversation_files(dir)?;
    let mut locks = vec![user_lock];
    let mut sessions = Vec::new();
    for p in &paths {
        locks.push(lock_file(p)?);
        sessions.push(Session::load(p)?);
    }
    let mut ctx = OnlineCtx {
        cli,
        http: reqwest::Client::new(),
        push: load_push(dir)?,
        store: DirStore { user: user.to_path_buf(), dir: dir.to_path_buf(), paths, locks, next_name: None },
    };
    let mut online = Online::new(keys, sessions, pow_bits);
    println!("address {}", address(&online.keys));
    if let Some(push) = &ctx.push {
        let all: Vec<&Session> = online.conversations.iter().collect();
        register_all(cli, &ctx.http, push, &online.keys, &all, true).await?;
        eprintln!("push token registered for the inbox and {} conversation(s)", all.len());
    }
    let mut stdin = Some(tokio::io::BufReader::new(tokio::io::stdin()).lines());
    let mut backoff = 1u64;
    loop {
        // každé připojení má nový efemerní klíč a novou trasu (failover i na jiné nody)
        let ek = SigningKey::generate(&mut OsRng);
        let connected = match fetch_route(&ctx.http, &cli.worker, &ek.verifying_key().to_bytes()).await {
            Ok(route) => open(cli, &ek, &route).await,
            Err(e) => Err(e.into()),
        };
        match connected {
            Ok((node, mut ws)) => {
                eprintln!("online via {}", node.hostname);
                let result = online_session(&mut ctx, &mut online, &mut ws, &mut stdin).await;
                ws.close().await;
                match result {
                    Ok(true) => return Ok(()),
                    Ok(false) => {}
                    Err(e) => eprintln!("connection lost: {e}"),
                }
                backoff = 1;
            }
            Err(e) => eprintln!("cannot connect: {e}"),
        }
        eprintln!("reconnecting in {backoff} s");
        tokio::select! {
            () = tokio::time::sleep(Duration::from_secs(backoff)) => {}
            _ = tokio::signal::ctrl_c() => return Ok(()),
        }
        backoff = (backoff * 2).min(60);
    }
}

async fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    if let Cmd::Init { file, role, secret } = &cli.command {
        let secret: [u8; 32] = match secret {
            Some(s) => b64u_decode_array(s).ok_or("secret must be 32 bytes in base64url")?,
            None => {
                let mut s = [0u8; 32];
                OsRng.fill_bytes(&mut s);
                s
            }
        };
        let role = match role {
            RoleArg::Initiator => Role::Initiator,
            RoleArg::Responder => Role::Responder,
        };
        Session::new(secret, role)?.save(file)?;
        println!("secret {}", b64u_encode(&secret));
        return Ok(());
    }
    match &cli.command {
        Cmd::Keygen { file } => {
            let keys = UserKeys::generate(now());
            create_user(&keys, file)?;
            println!("address {}", address(&keys));
            return Ok(());
        }
        Cmd::Safety { user, address } => {
            let pk = parse_address(address)?;
            println!("{}", safety(&load_user(user)?, &pk));
            return Ok(());
        }
        Cmd::Whoami { file, qr } => {
            let text = address(&load_user(file)?);
            if *qr {
                let code = qrcode::QrCode::new(text.as_bytes())?;
                println!("{}", code.render::<qrcode::render::unicode::Dense1x2>().quiet_zone(true).build());
            }
            println!("{text}");
            return Ok(());
        }
        Cmd::Online { user, dir, pow_bits } => return run_online(&cli, user, dir, *pow_bits).await,
        Cmd::PushRegister { user, dir, platform, token } => {
            let keys = load_user(user)?;
            std::fs::create_dir_all(dir)?;
            let push = PushConfig { platform: platform.clone(), token: token.clone() };
            write_private(&dir.join(PUSH_FILE), &serde_json::to_string_pretty(&push)?, false)?;
            let sessions = conversation_files(dir)?.iter().map(|p| Session::load(p)).collect::<Result<Vec<_>, _>>()?;
            let all: Vec<&Session> = sessions.iter().collect();
            register_all(&cli, &reqwest::Client::new(), &push, &keys, &all, true).await?;
            println!("push token registered for the inbox and {} conversation(s)", all.len());
            return Ok(());
        }
        Cmd::History { file } => {
            for h in &load_session(file)?.history {
                println!("{} {} {}", h.ts, if h.outgoing { ">" } else { "<" }, h.text);
            }
            return Ok(());
        }
        _ => {}
    }
    // zámek dřív než síť: druhý proces nad stejným stavem skončí hned (viz `lock_file`)
    let _lock = match &cli.command {
        Cmd::Publish { file, .. } | Cmd::Typing { file, .. } | Cmd::Status { file, .. } | Cmd::Send { file, .. } | Cmd::Recv { file } | Cmd::Watch { file } => {
            Some(lock_file(file)?)
        }
        Cmd::Start { user, .. } | Cmd::Inbox { user, .. } => Some(lock_file(user)?),
        _ => None,
    };
    let ek = SigningKey::generate(&mut OsRng);
    let http = reqwest::Client::new();
    let route = fetch_route(&http, &cli.worker, &ek.verifying_key().to_bytes()).await?;
    match &cli.command {
        Cmd::Route => {
            println!("epoch {} (ticket ttl {} s)", route.epoch, route.ttl);
            for n in &route.nodes {
                println!("{}  {}", n.node_id, n.hostname);
            }
        }
        Cmd::Connect => {
            let (node, mut ws) = open(&cli, &ek, &route).await?;
            println!("connected to {} ({})", node.node_id, node.hostname);
            ws.send_json(&serde_json::json!({"t": "ping"})).await?;
            println!("{}", ws.recv_json().await?);
            ws.close().await;
        }
        Cmd::Publish { file, pow_bits } => {
            let mut keys = load_user(file)?;
            // rotace prekey a doplnění jednorázových: tajné části se uloží dřív, než je profil zveřejní
            if keys.maintain(now()) {
                save_user(&keys, file)?;
            }
            let (_, mut ws) = open(&cli, &ek, &route).await?;
            let acks = publish_profile(&mut ws, &keys, *pow_bits).await?;
            println!("profile stored on {acks} anchors");
            ws.close().await;
        }
        Cmd::Lookup { address } => {
            let pk = parse_address(address)?;
            let (_, mut ws) = open(&cli, &ek, &route).await?;
            let profile = lookup_profile(&mut ws, &pk).await;
            ws.close().await;
            let profile = profile?;
            println!("address {}\nbox key {}\nupdated {}", b64u_encode(&profile.pk), b64u_encode(&profile.bk), profile.ts);
        }
        Cmd::Start { user, address, conversation } => {
            if conversation.exists() {
                return Err(format!("{} already exists; refusing to overwrite a ratchet state", conversation.display()).into());
            }
            let keys = load_user(user)?;
            let pk = parse_address(address)?;
            let (_, mut ws) = open(&cli, &ek, &route).await?;
            let session = start_conversation(&mut ws, &keys, &pk).await;
            ws.close().await;
            session?.save(conversation)?;
            println!("conversation saved to {}", conversation.display());
            println!("safety number {}", safety(&keys, &pk));
        }
        Cmd::Inbox { user, dir } => {
            let mut keys = load_user(user)?;
            std::fs::create_dir_all(dir)?;
            let (_, mut ws) = open(&cli, &ek, &route).await?;
            let batch = process_inbox(&mut ws, &mut keys).await;
            let batch = match batch {
                Ok(b) => b,
                Err(e) => {
                    ws.close().await;
                    return Err(e.into());
                }
            };
            // nejdřív uložit konverzace a klíče bez použitých prekeys, teprve potom potvrdit obálky
            for a in &batch.accepted {
                let file = dir.join(format!("{}.json", b64u_encode(&a.peer)));
                if file.exists() {
                    eprintln!("{} already exists, keeping the existing conversation", file.display());
                    continue;
                }
                a.session.save(&file)?;
                println!("new conversation with {}\nsafety number {}", b64u_encode(&a.peer), safety(&keys, &a.peer));
            }
            save_user(&keys, user)?;
            ack_inbox(&mut ws, &keys, &batch.done).await?;
            if !batch.accepted.is_empty() {
                eprintln!("used one-time prekeys were deleted; run `publish` so that others do not pick them");
            }
            if batch.postponed > 0 {
                eprintln!("{} handshake(s) postponed (sender profile not found yet)", batch.postponed);
            }
            ws.close().await;
        }
        Cmd::Typing { file, off } => {
            let session = load_session(file)?;
            let (_, mut ws) = open(&cli, &ek, &route).await?;
            send_typing(&mut ws, &session, !off).await?;
            ws.close().await;
        }
        Cmd::Status { file, ids } => {
            let session = load_session(file)?;
            let parsed: Vec<[u8; 16]> = ids.iter().map(|i| b64u_decode_array(i).ok_or("message id must be 16 bytes in base64url")).collect::<Result<_, _>>()?;
            let (_, mut ws) = open(&cli, &ek, &route).await?;
            let states = message_status(&mut ws, &session, &parsed).await;
            ws.close().await;
            for (id, state) in ids.iter().zip(states?) {
                println!("{id} {state}");
            }
        }
        Cmd::Send { file, text } => {
            let mut session = load_session(file)?;
            let prepared = session.prepare(text.as_bytes())?;
            session.save(file)?;
            let (node, mut ws) = open(&cli, &ek, &route).await?;
            let acks = transmit(&mut ws, &prepared).await?;
            println!("sent via {} ({acks} anchors confirmed), id {}", node.hostname, b64u_encode(&prepared.id));
            ws.close().await;
        }
        Cmd::Recv { file } => {
            let mut session = load_session(file)?;
            let (_, mut ws) = open(&cli, &ek, &route).await?;
            let received = receive(&mut ws, &mut session).await;
            // stav ratchetu se ukládá i po chybě sítě: dešifrované zprávy už jsou „spotřebované“
            session.save(file)?;
            let received = received?;
            for m in &received.messages {
                println!("{}", String::from_utf8_lossy(m));
            }
            if received.rejected > 0 {
                eprintln!("{} message(s) could not be decrypted", received.rejected);
            }
            ws.close().await;
        }
        Cmd::Watch { file } => {
            let mut session = load_session(file)?;
            let (node, mut ws) = open(&cli, &ek, &route).await?;
            eprintln!("watching via {}", node.hostname);
            // nejdřív odběr (anchory pak posílají nové zprávy rovnou), pak zprávy, které čekaly offline
            subscribe(&mut ws, &session).await?;
            let backlog = receive(&mut ws, &mut session).await;
            session.save(file)?;
            for m in &backlog?.messages {
                println!("{}", String::from_utf8_lossy(m));
            }
            loop {
                let pushed = tokio::select! {
                    p = next_event(&mut ws, &mut session, std::time::Duration::from_secs(25)) => p,
                    _ = tokio::signal::ctrl_c() => break,
                };
                // stav ratchetu se ukládá po každé zprávě
                session.save(file)?;
                match pushed? {
                    Some(Event::Message(m)) => println!("{}", String::from_utf8_lossy(&m)),
                    Some(Event::Typing(on)) => eprintln!("{}", if on { "(typing…)" } else { "(stopped typing)" }),
                    None => ws.send_json(&serde_json::json!({"t": "ping"})).await?,
                }
            }
            ws.close().await;
        }
        Cmd::Init { .. }
        | Cmd::Keygen { .. }
        | Cmd::Whoami { .. }
        | Cmd::History { .. }
        | Cmd::Safety { .. }
        | Cmd::Online { .. }
        | Cmd::PushRegister { .. } => {}
    }
    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    match run(Cli::parse()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
