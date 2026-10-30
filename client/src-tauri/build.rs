use tauri_build::{AppManifest, Attributes};

/// Příkazy UI → Rust (§13.5). Musí odpovídat `COMMAND_NAMES` v `src/lib/types.ts`; hlídá to `client/test/contract.test.ts`.
const COMMANDS: &[&str] = &[
    "identity_status", "identity_create", "address_get", "address_qr",
    "contact_add", "contact_list", "contact_safety_number", "contact_mark_verified",
    "conversation_list", "conversation_open", "message_send", "typing_set", "conversation_delete",
    "connection_state", "settings_get", "settings_set", "lock", "unlock",
    "biometric_status", "biometric_enable", "biometric_disable", "unlock_biometric",
];

fn main() {
    // Bez `AppManifest` jsou vlastní příkazy dostupné vždy. Takto se musí každý povolit v capabilities.
    tauri_build::try_build(Attributes::new().app_manifest(AppManifest::new().commands(COMMANDS)))
        .unwrap_or_else(|e| panic!("tauri build selhal: {e}"));
}
