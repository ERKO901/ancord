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

use crate::core::BiometricKey;
use crate::store::{DeviceKey, StoreError};

struct Bridge {
    vm: JavaVM,
    class: GlobalRef,
    bio: GlobalRef,
}

static BRIDGE: OnceLock<Bridge> = OnceLock::new();

#[allow(unsafe_code, reason = "JNI_OnLoad je vstupní bod knihovny (§13.4)")]
#[no_mangle]
pub extern "system" fn JNI_OnLoad(vm: JavaVM, _reserved: *mut c_void) -> jint {
    let classes = vm.get_env().ok().and_then(|mut env| {
        let class = env.find_class("cz/melonet/ancord/KeystoreKey").ok()?;
        let class = env.new_global_ref(class).ok()?;
        let bio = env.find_class("cz/melonet/ancord/BiometricKey").ok()?;
        Some((class, env.new_global_ref(bio).ok()?))
    });
    if let Some((class, bio)) = classes {
        let _ = BRIDGE.set(Bridge { vm, class, bio });
    }
    JNI_VERSION_1_6
}

fn io(msg: &str) -> StoreError {
    StoreError::Io(msg.to_owned())
}

/// Zavolá `KeystoreKey.<method>(byte[]) -> byte[]?`.
fn call(method: &str, input: &[u8]) -> Result<Vec<u8>, StoreError> {
    call_on(false, method, input)
}

#[allow(unsafe_code, reason = "JClass z globální reference, viz JNI_OnLoad")]
fn call_on(bio: bool, method: &str, input: &[u8]) -> Result<Vec<u8>, StoreError> {
    let bridge = BRIDGE.get().ok_or_else(|| io("Keystore není k dispozici"))?;
    let mut env = bridge.vm.attach_current_thread().map_err(|_| io("JNI"))?;
    let arg = env.byte_array_from_slice(input).map_err(|_| io("JNI"))?;
    // SAFETY: globální reference míří na `java.lang.Class` a žije po celou dobu procesu.
    let class = unsafe { JClass::from_raw(if bio { bridge.bio.as_obj().as_raw() } else { bridge.class.as_obj().as_raw() }) };
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

/// Biometrická obálka hlavního klíče (Kotlin `BiometricKey`): `wrap`/`unwrap` zobrazí systémový dotaz na otisk nebo obličej.
pub struct AndroidBiometric;

impl BiometricKey for AndroidBiometric {
    fn available(&self) -> bool {
        let Some(bridge) = BRIDGE.get() else { return false };
        let Ok(mut env) = bridge.vm.attach_current_thread() else { return false };
        // SAFETY: globální reference míří na `java.lang.Class` a žije po celou dobu procesu.
        #[allow(unsafe_code, reason = "JClass z globální reference, viz JNI_OnLoad")]
        let class = unsafe { JClass::from_raw(bridge.bio.as_obj().as_raw()) };
        let ok = env.call_static_method(&class, "available", "()Z", &[]).and_then(|v| v.z()).unwrap_or(false);
        let _ = env.exception_clear();
        ok
    }

    fn wrap(&self, plain: &[u8]) -> Result<Vec<u8>, String> {
        call_on(true, "wrap", plain).map_err(|_| "Biometrika byla zrušena nebo selhala".to_string())
    }

    fn unwrap(&self, blob: &[u8]) -> Result<Zeroizing<Vec<u8>>, String> {
        call_on(true, "unwrap", blob)
            .map(Zeroizing::new)
            .map_err(|_| "Biometrika byla zrušena nebo už neplatí (nový otisk?). Odemkněte heslem.".to_string())
    }

    fn forget(&self) {
        let Some(bridge) = BRIDGE.get() else { return };
        let Ok(mut env) = bridge.vm.attach_current_thread() else { return };
        #[allow(unsafe_code, reason = "JClass z globální reference, viz JNI_OnLoad")]
        // SAFETY: viz výše.
        let class = unsafe { JClass::from_raw(bridge.bio.as_obj().as_raw()) };
        let _ = env.call_static_method(&class, "forget", "()V", &[]);
        let _ = env.exception_clear();
    }
}
