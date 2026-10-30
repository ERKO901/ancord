//! Sdílené jádro nodu a klienta (`ARCHITECTURE.md` §13.4): protokolové primitiva, identita uživatele,
//! Double Ratchet, šifrovaný kanál, profil a stráž schránky. Žádné serverové závislosti (HTTP server, redb, sysinfo).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

pub mod canonical;
pub mod channel;
pub mod encoding;
pub mod error;
pub mod guard;
pub mod hashcash;
pub mod hrw;
pub mod net;
pub mod online;
pub mod profile;
pub mod ratchet;
pub mod sign;
pub mod ticket;
pub mod time;
pub mod user;
pub mod wake;
