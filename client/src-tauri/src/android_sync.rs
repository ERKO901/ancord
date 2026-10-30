//! D6, prototyp 13.10: `sync_once` z `BroadcastReceiveru` bez Activity a WebView.
//! Vstup volá Kotlin `SyncReceiver` po probuzení; profil se otevře ze složky dat, žádný Tauri runtime se nespouští.

use std::sync::Arc;
use std::time::Duration;

use jni::objects::{JClass, JString};
use jni::sys::jint;
use jni::JNIEnv;

use crate::contract::UiEvent;
use crate::core::Core;
use crate::net::{NetConfig, Sink};
use crate::store::KdfParams;

/// Na pozadí není komu události ukázat; oznámení bez obsahu řeší Kotlin podle návratové hodnoty.
struct LogSink;

impl Sink for LogSink {
    fn emit(&self, event: UiEvent) {
        eprintln!("ancord sync: {}", event.name());
    }
}

fn run(dir: String) -> Result<usize, String> {
    let config = NetConfig::from_build().ok_or("chybí konfigurace Workeru")?;
    // Android nemá Keychain, profil s heslem zůstane zamčený a `sync_once` ho neotevře (zprávy se vyzvednou po odemknutí).
    let core = Core::persistent(dir.into(), KdfParams::default(), false)?;
    core.sync_once(&config, Arc::new(LogSink), Duration::from_secs(20))
}

/// Vrací počet přijatých zpráv, `-1` při chybě (text jde do logcatu).
#[allow(unsafe_code, reason = "JNI export, jediné místo s unsafe (§13.4)")]
#[no_mangle]
pub extern "system" fn Java_cz_melonet_ancord_SyncReceiver_nativeSync(mut env: JNIEnv, _class: JClass, dir: JString) -> jint {
    let Ok(dir) = env.get_string(&dir).map(String::from) else { return -1 };
    match run(dir) {
        Ok(n) => jint::try_from(n).unwrap_or(jint::MAX),
        Err(e) => {
            eprintln!("ancord sync: {e}");
            -1
        }
    }
}
