//! ancord node: identita, registrace u Workeru, heartbeat, odpovědi na challenge a admission klientů.
//! Protokol je v `spec/PROTOCOL.md`; sdílí se s Workerem jen specifikace a testovací vektory.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

pub mod canonical;
pub mod challenge;
pub mod client;
pub mod config;
pub mod control;
pub mod encoding;
pub mod error;
pub mod hashcash;
pub mod identity;
pub mod register;
pub mod server;
pub mod sign;
pub mod state;
pub mod supervisor;
pub mod sysinfo;
pub mod ticket;
pub mod time;
pub mod tunnel;
pub mod worker;
