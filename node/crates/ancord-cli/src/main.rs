use std::process::ExitCode;

use ancord_cli::{connect_any, default_url, fetch_route};
use clap::{Parser, Subcommand};
use ed25519_dalek::SigningKey;
use futures_util::{SinkExt, StreamExt};
use rand::rngs::OsRng;
use tokio_tungstenite::tungstenite::Message;

#[derive(Parser)]
#[command(name = "ancord", about = "ancord CLI klient")]
struct Cli {
    /// Adresa Workeru.
    #[arg(long, env = "WORKER_URL")]
    worker: String,
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Vypíše nody z `/route` (s novým efemerním klíčem).
    Route,
    /// Připojí se na první node, který klienta přijme, a pošle ping.
    Connect {
        /// Šablona adresy nodu; `{hostname}` se nahradí. Výchozí `wss://{hostname}/client`.
        #[arg(long)]
        via: Option<String>,
    },
}

async fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let ek = SigningKey::generate(&mut OsRng);
    let http = reqwest::Client::new();
    let route = fetch_route(&http, &cli.worker, &ek.verifying_key().to_bytes()).await?;
    match cli.command {
        Cmd::Route => {
            println!("epoch {} (ticket ttl {} s)", route.epoch, route.ttl);
            for n in &route.nodes {
                println!("{}  {}", n.node_id, n.hostname);
            }
        }
        Cmd::Connect { via } => {
            let url_for = |host: &str| via.as_ref().map_or_else(|| default_url(host), |t| t.replace("{hostname}", host));
            let (node, mut ws) = connect_any(&route, &ek, url_for).await?;
            println!("connected to {} ({})", node.node_id, node.hostname);
            ws.send(Message::text(r#"{"t":"ping"}"#)).await?;
            if let Some(Ok(Message::Text(t))) = ws.next().await {
                println!("{t}");
            }
            let _ = ws.close(None).await;
        }
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
