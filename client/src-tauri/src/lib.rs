//! Rust shell klienta (§13.3). WebView je nedůvěryhodný: smí volat jen příkazy z kontraktu.

#[cfg(target_os = "android")]
mod android_keystore;
#[cfg(target_os = "android")]
mod android_sync;
mod contract;
mod core;
#[cfg(test)]
mod e2e;
mod net;
mod store;

use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};

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

/// Ochrana obsahu okna před snímáním obrazovky (§13.9). Na Linuxu ji platforma neumožňuje a volání nic nedělá.
fn apply_protection(window: &WebviewWindow, on: bool) {
    let _ = window.set_content_protected(on);
}

#[tauri::command]
fn settings_set(core: State<Core>, window: WebviewWindow, settings: Settings) -> Result<(), String> {
    let on = settings.enhanced_protection;
    core.settings_set(settings)?;
    apply_protection(&window, on);
    Ok(())
}

#[tauri::command]
fn lock(core: State<Core>) -> Result<(), String> {
    core.lock()
}

#[tauri::command]
fn unlock(core: State<Core>, window: WebviewWindow, passphrase: Option<String>) -> Result<(), String> {
    core.unlock(passphrase)?;
    if let Ok(settings) = core.settings_get() {
        apply_protection(&window, settings.enhanced_protection);
    }
    Ok(())
}

/// Události pro UI (`Events` v `types.ts`).
struct AppSink(AppHandle);

impl Sink for AppSink {
    fn emit(&self, event: UiEvent) {
        let _ = self.0.emit(event.name(), &event);
    }
}

/// Proces nesmí vyrobit core dump ani být čten přes ptrace/`/proc/<pid>/mem` od jiného procesu téhož uživatele (§9 bod 16).
/// Na Linuxu `PR_SET_DUMPABLE=0`; Windows (WER) a macOS (crash reporter) řeší packaging, viz `client/README.md`.
fn disable_core_dumps() {
    #[cfg(target_os = "linux")]
    if let Err(code) = prctl::set_dumpable(false) {
        eprintln!("ancord: PR_SET_DUMPABLE selhal ({code})");
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    disable_core_dumps();
    let builder = tauri::Builder::default();
    // Druhé spuštění jen vynese okno prvního; dva procesy nad jedním profilem by si lámaly stav ratchetu (§13.9).
    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.unminimize();
            let _ = window.show();
            let _ = window.set_focus();
        }
    }));
    let result = builder
        .setup(|app| {
            // Profil v datové složce aplikace; chyba otevření (poškozený soubor) aplikaci nespustí, nic se nepřepíše.
            let dir = app.path().app_data_dir()?;
            let core = Core::persistent(dir, KdfParams::default(), true)?;
            // bez zapečené adresy a klíče Workeru (§13.3) zůstane klient offline
            if let Some(config) = NetConfig::from_build() {
                core.go_online(config, Arc::new(AppSink(app.handle().clone())))?;
            }
            // profil zamčený heslem nemá nastavení k dispozici, proto výchozí je ochrana zapnutá; po odemčení se přepíše
            let protect = core.settings_get().map(|s| s.enhanced_protection).unwrap_or(true);
            if let Some(window) = app.get_webview_window("main") {
                apply_protection(&window, protect);
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

/// Kontrola závislostí (§13.9, D5): žádná telemetrie, crash reporter, updater ani plugin s přístupem k síti/souborům z WebView.
#[cfg(test)]
mod deps_audit {
    const FORBIDDEN: &[&str] = &[
        "sentry", "sentry-core", "minidumper", "crash-handler", "breakpad", "firebase", "posthog-rs", "segment",
        "tauri-plugin-updater", "tauri-plugin-http", "tauri-plugin-shell", "tauri-plugin-fs", "tauri-plugin-opener",
        "tauri-plugin-global-shortcut", "tauri-plugin-log",
    ];

    #[test]
    fn lockfile_has_no_telemetry_or_ambient_plugins() {
        let lock = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.lock")).unwrap_or_default();
        assert!(lock.contains("name = \"tauri\""), "Cargo.lock se nenačetl");
        let found: Vec<&str> = lock
            .lines()
            .filter_map(|l| l.strip_prefix("name = \"").and_then(|l| l.strip_suffix('"')))
            .filter(|n| FORBIDDEN.contains(n))
            .collect();
        assert!(found.is_empty(), "zakázané závislosti: {found:?}");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn core_dumps_are_disabled() {
        super::disable_core_dumps();
        assert_eq!(prctl::get_dumpable(), Ok(false));
    }
}
