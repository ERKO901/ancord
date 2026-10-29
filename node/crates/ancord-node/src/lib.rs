//! ancord node: identita, registrace u Workeru, heartbeat, odpovědi na challenge a admission klientů.
//! Protokol je v `spec/PROTOCOL.md`; sdílí se s Workerem jen specifikace a testovací vektory.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

pub mod canonical;
pub mod challenge;
pub mod channel;
pub mod client;
pub mod config;
pub mod control;
pub mod encoding;
pub mod data;
pub mod error;
pub mod hashcash;
pub mod hrw;
pub mod identity;
pub mod maintenance;
pub mod n2n;
pub mod peer;
pub mod presence;
pub mod profile;
pub mod ratchet;
pub mod register;
pub mod server;
pub mod sign;
pub mod state;
pub mod store;
pub mod supervisor;
pub mod sysinfo;
pub mod ticket;
pub mod time;
pub mod user;
pub mod tunnel;
pub mod wake;
pub mod worker;
