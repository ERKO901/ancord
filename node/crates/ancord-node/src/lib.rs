//! ancord node: identita, registrace u Workeru, heartbeat, odpovědi na challenge a admission klientů.
//! Protokol je v `spec/PROTOCOL.md`; sdílí se s Workerem jen specifikace a testovací vektory.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

pub mod challenge;
pub mod client;
pub use ancord_core::{canonical, channel, encoding, hashcash, hrw, ratchet, sign, ticket, time, user};

pub mod config;
pub mod control;
pub mod data;
pub mod error;
pub mod guard;
pub mod identity;
pub mod maintenance;
pub mod n2n;
pub mod peer;
pub mod presence;
pub mod reach;
pub mod profile;
pub mod register;
pub mod server;
pub mod state;
pub mod store;
pub mod supervisor;
pub mod sysinfo;
pub mod tunnel;
pub mod wake;
pub mod worker;
