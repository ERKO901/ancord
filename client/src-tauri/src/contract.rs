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

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum DeliveryState {
    Queued,
    Pending,
    Delivered,
    Unknown,
}

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionState {
    #[default]
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

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
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

/// Proč se klient nepřipojil nebo spojení nefunguje (`null` = žádný problém). Rozlišení je hrubé a jen z textu chyb,
/// UI z něj dostane srozumitelný důvod, ne adresy ani tickety.
#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionIssue {
    /// Worker nedostupný nebo bez sítě.
    WorkerUnreachable,
    /// Worker odpověděl chybou nebo neočekávanou odpovědí.
    WorkerError,
    /// Trasa nebo ticket se nepodařilo ověřit klíčem Workeru.
    RouteUntrusted,
    /// Žádný node z trasy se nepodařilo navázat.
    NodeUnreachable,
    /// Node odmítl požadavek klienta (nejspíš nesoulad verzí protokolu).
    NodeRejected,
}

/// Události Rust → UI (`Events` v `types.ts`). Název události je `UiEvent::name`, tělo je varianta bez obalu.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum UiEvent {
    #[serde(rename_all = "camelCase")]
    MessageReceived { conversation_id: String, message: Message },
    #[serde(rename_all = "camelCase")]
    DeliveryState { conversation_id: String, message_id: String, state: DeliveryState },
    #[serde(rename_all = "camelCase")]
    Typing { conversation_id: String, on: bool },
    ConnectionState { state: ConnectionState },
    ContactRequest { contact: Contact },
    ConnectionIssue { issue: Option<ConnectionIssue> },
}

impl UiEvent {
    pub fn name(&self) -> &'static str {
        match self {
            UiEvent::MessageReceived { .. } => "message_received",
            UiEvent::DeliveryState { .. } => "delivery_state",
            UiEvent::Typing { .. } => "typing",
            UiEvent::ConnectionState { .. } => "connection_state",
            UiEvent::ContactRequest { .. } => "contact_request",
            UiEvent::ConnectionIssue { .. } => "connection_issue",
        }
    }
}
