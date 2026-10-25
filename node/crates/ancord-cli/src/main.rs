use std::process::ExitCode;

use std::path::{Path, PathBuf};

use ancord_cli::{connect_any, default_url, fetch_route, next_pushed, receive, subscribe, transmit, NodeSocket, Role, Route, RouteNode, Session};
use ancord_node::encoding::{b64u_decode_array, b64u_encode};
use clap::{Parser, Subcommand, ValueEnum};
use rand::RngCore;
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;

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
        Session::new(secret, role).save(file)?;
        println!("secret {}", b64u_encode(&secret));
        return Ok(());
    }
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
        Cmd::Send { file, text } => {
            let mut session = load_session(file)?;
            let prepared = session.prepare(text.as_bytes())?;
            session.save(file)?;
            let (node, mut ws) = open(&cli, &ek, &route).await?;
            let acks = transmit(&mut ws, &prepared).await?;
            println!("sent via {} ({acks} anchors confirmed)", node.hostname);
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
                    p = next_pushed(&mut ws, &mut session, std::time::Duration::from_secs(25)) => p,
                    _ = tokio::signal::ctrl_c() => break,
                };
                // stav ratchetu se ukládá po každé zprávě
                session.save(file)?;
                match pushed? {
                    Some(m) => println!("{}", String::from_utf8_lossy(&m)),
                    None => ws.send_json(&serde_json::json!({"t": "ping"})).await?,
                }
            }
            ws.close().await;
        }
        Cmd::Init { .. } => {}
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
