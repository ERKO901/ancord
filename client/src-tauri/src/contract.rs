//! IPC kontrakt UI ↔ Rust (§13.5). Zrcadlí `client/src/lib/types.ts`; názvy polí jsou camelCase.
//! UI dostane jen to, co zobrazuje: žádné klíče, tickety ani adresy nodů.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum IdentityState {
    None,
    Locked,
    Ready,
}

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)] // varianty použije online klient (D3)
pub enum DeliveryState {
    Queued,
    Pending,
    Delivered,
    Unknown,
}

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)] // varianty použije online klient (D3)
pub enum ConnectionState {
    Offline,
    Connecting,
    Online,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ThemeChoice {
    System,
    Dark,
    Light,
}

#[derive(Serialize, Clone, Copy, Debug)]
pub struct IdentityStatus {
    pub state: IdentityState,
    pub passphrase: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct Contact {
    pub id: String,
    pub name: String,
    pub address: String,
    pub verified: bool,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConversationSummary {
    pub id: String,
    pub contact_id: String,
    pub name: String,
    pub last_text: String,
    pub last_ts: u64,
    pub unread: u32,
    pub last_state: Option<DeliveryState>,
}

#[derive(Serialize, Clone, Debug)]
pub struct Message {
    pub id: String,
    pub ts: u64,
    pub mine: bool,
    pub text: String,
    pub state: DeliveryState,
}

#[derive(Serialize, Clone, Debug)]
pub struct ConversationPage {
    pub messages: Vec<Message>,
    pub more: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub theme: ThemeChoice,
    pub enhanced_protection: bool,
    pub lock_after_secs: u32,
    pub notification_preview: bool,
    pub keep_connection: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { theme: ThemeChoice::Dark, enhanced_protection: true, lock_after_secs: 300, notification_preview: false, keep_connection: false }
    }
}
