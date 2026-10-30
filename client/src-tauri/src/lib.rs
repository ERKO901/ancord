//! Rust shell klienta (§13.3). WebView je nedůvěryhodný: smí volat jen příkazy z kontraktu.

mod contract;
mod core;
#[cfg(test)]
mod e2e;
mod net;
mod store;

use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager, State};

use crate::contract::{
    ConnectionState, Contact, ConversationPage, ConversationSummary, IdentityStatus, Message, Settings, UiEvent,
};
use crate::core::Core;
use crate::net::{NetConfig, Sink};
use crate::store::KdfParams;

#[tauri::command]
fn identity_status(core: State<Core>) -> Result<IdentityStatus, String> {
    core.identity_status()
}

#[tauri::command]
fn identity_create(core: State<Core>, passphrase: Option<String>) -> Result<String, String> {
    core.identity_create(passphrase)
}

#[tauri::command]
fn address_get(core: State<Core>) -> Result<String, String> {
    core.address_get()
}

#[tauri::command]
fn address_qr(core: State<Core>) -> Result<String, String> {
    core.address_qr()
}

#[tauri::command]
fn contact_add(core: State<Core>, address: String, name: Option<String>) -> Result<Contact, String> {
    core.contact_add(&address, name)
}

#[tauri::command]
fn contact_list(core: State<Core>) -> Result<Vec<Contact>, String> {
    core.contact_list()
}

#[tauri::command]
fn contact_safety_number(core: State<Core>, id: String) -> Result<String, String> {
    core.contact_safety_number(&id)
}

#[tauri::command]
fn contact_mark_verified(core: State<Core>, id: String) -> Result<(), String> {
    core.contact_mark_verified(&id)
}

#[tauri::command]
fn conversation_list(core: State<Core>) -> Result<Vec<ConversationSummary>, String> {
    core.conversation_list()
}

#[tauri::command]
fn conversation_open(core: State<Core>, id: String, before: Option<u64>, limit: u32) -> Result<ConversationPage, String> {
    core.conversation_open(&id, before, limit)
}

#[tauri::command]
fn message_send(core: State<Core>, id: String, text: String) -> Result<Message, String> {
    core.message_send(&id, &text)
}

#[tauri::command]
fn typing_set(core: State<Core>, id: String, on: bool) {
    core.typing_set(&id, on);
}

#[tauri::command]
fn conversation_delete(core: State<Core>, id: String) -> Result<(), String> {
    core.conversation_delete(&id)
}

#[tauri::command]
fn connection_state(core: State<Core>) -> ConnectionState {
    core.connection_state()
}

#[tauri::command]
fn settings_get(core: State<Core>) -> Result<Settings, String> {
    core.settings_get()
}

#[tauri::command]
fn settings_set(core: State<Core>, settings: Settings) -> Result<(), String> {
    core.settings_set(settings)
}

#[tauri::command]
fn lock(core: State<Core>) -> Result<(), String> {
    core.lock()
}

#[tauri::command]
fn unlock(core: State<Core>, passphrase: Option<String>) -> Result<(), String> {
    core.unlock(passphrase)
}

/// Události pro UI (`Events` v `types.ts`).
struct AppSink(AppHandle);

impl Sink for AppSink {
    fn emit(&self, event: UiEvent) {
        let _ = self.0.emit(event.name(), &event);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let result = tauri::Builder::default()
        .setup(|app| {
            // Profil v datové složce aplikace; chyba otevření (poškozený soubor) aplikaci nespustí, nic se nepřepíše.
            let dir = app.path().app_data_dir()?;
            let core = Core::persistent(dir, KdfParams::default(), true)?;
            // bez zapečené adresy a klíče Workeru (§13.3) zůstane klient offline
            if let Some(config) = NetConfig::from_build() {
                core.go_online(config, Arc::new(AppSink(app.handle().clone())))?;
            }
            app.manage(core);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            identity_status, identity_create, address_get, address_qr,
            contact_add, contact_list, contact_safety_number, contact_mark_verified,
            conversation_list, conversation_open, message_send, typing_set, conversation_delete,
            connection_state, settings_get, settings_set, lock, unlock,
        ])
        .run(tauri::generate_context!());
    if let Err(e) = result {
        eprintln!("ancord: aplikaci se nepodařilo spustit: {e}");
        std::process::exit(1);
    }
}
