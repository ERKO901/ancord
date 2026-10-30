//! D6: klíč zařízení zabalený Android Keystore (Kotlin `KeystoreKey`) přes JNI.
//! Třídu si `JNI_OnLoad` uloží jako globální referenci: z vláken vytvořených Rustem by `FindClass` aplikační třídu nenašel.

use std::ffi::c_void;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;

use jni::objects::{GlobalRef, JByteArray, JClass, JValue};
use jni::sys::{jint, JNI_VERSION_1_6};
use jni::JavaVM;
use rand::RngCore;
use zeroize::Zeroizing;

use crate::store::{DeviceKey, StoreError};

struct Bridge {
    vm: JavaVM,
    class: GlobalRef,
}

static BRIDGE: OnceLock<Bridge> = OnceLock::new();

#[allow(unsafe_code, reason = "JNI_OnLoad je vstupní bod knihovny (§13.4)")]
#[no_mangle]
pub extern "system" fn JNI_OnLoad(vm: JavaVM, _reserved: *mut c_void) -> jint {
    let class = vm.get_env().ok().and_then(|mut env| {
        let class = env.find_class("cz/melonet/ancord/KeystoreKey").ok()?;
        env.new_global_ref(class).ok()
    });
    if let Some(class) = class {
        let _ = BRIDGE.set(Bridge { vm, class });
    }
    JNI_VERSION_1_6
}

fn io(msg: &str) -> StoreError {
    StoreError::Io(msg.to_owned())
}

/// Zavolá `KeystoreKey.<method>(byte[]) -> byte[]?`.
#[allow(unsafe_code, reason = "JClass z globální reference, viz JNI_OnLoad")]
fn call(method: &str, input: &[u8]) -> Result<Vec<u8>, StoreError> {
    let bridge = BRIDGE.get().ok_or_else(|| io("Keystore není k dispozici"))?;
    let mut env = bridge.vm.attach_current_thread().map_err(|_| io("JNI"))?;
    let arg = env.byte_array_from_slice(input).map_err(|_| io("JNI"))?;
    // SAFETY: globální reference míří na `java.lang.Class` a žije po celou dobu procesu.
    let class = unsafe { JClass::from_raw(bridge.class.as_obj().as_raw()) };
    let out = env
        .call_static_method(&class, method, "([B)[B", &[JValue::Object(&arg)])
        .and_then(|v| v.l())
        .map_err(|_| {
            // Nevyřízená Java výjimka by se vrátila do Javy z nejbližšího JNI volání (např. `Ipc.postMessage`).
            let _ = env.exception_clear();
            io("Keystore selhal")
        })?;
    if out.is_null() {
        return Err(io("Keystore odmítl operaci"));
    }
    env.convert_byte_array(JByteArray::from(out)).map_err(|_| io("JNI"))
}

/// Klíč zařízení: náhodných 32 B zabalených klíčem z Keystore, zabalený blob leží v `path`.
/// Bez klíče v Keystore (přeinstalace, obnova ze zálohy) se blob nerozbalí a profil se neotevře.
pub struct AndroidKeystoreDeviceKey(pub PathBuf);

impl DeviceKey for AndroidKeystoreDeviceKey {
    fn load_or_create(&self) -> Result<Zeroizing<[u8; 32]>, StoreError> {
        match fs::read(&self.0) {
            Ok(blob) => {
                let plain = Zeroizing::new(call("unwrap", &blob)?);
                let key: [u8; 32] = plain.as_slice().try_into().map_err(|_| StoreError::Unlock)?;
                Ok(Zeroizing::new(key))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let mut key = Zeroizing::new([0u8; 32]);
                rand::rngs::OsRng.fill_bytes(key.as_mut());
                let blob = call("wrap", key.as_ref())?;
                let mut f = fs::OpenOptions::new().write(true).create_new(true).open(&self.0).map_err(|e| StoreError::Io(e.to_string()))?;
                f.write_all(&blob).and_then(|()| f.sync_all()).map_err(|e| StoreError::Io(e.to_string()))?;
                Ok(key)
            }
            Err(e) => Err(StoreError::Io(e.to_string())),
        }
    }
}
