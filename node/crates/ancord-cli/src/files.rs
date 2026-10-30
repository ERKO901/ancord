//! Souborové úložiště CLI: klíče a konverzace v JSON souborech s právy 0600 a zámkem (jen Unix).
//! Klient (Tauri) má vlastní šifrované úložiště; sdílené jádro souborové operace nezná.

use ancord_core::net::{CliError, Session};
use ancord_core::user::UserKeys;

/// Zapíše soubor s právy 0600; `create_new` existující nepřepíše, jinak atomicky přes dočasný soubor.
pub fn write_private(path: &std::path::Path, text: &str, create_new: bool) -> Result<(), CliError> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let target = if create_new { path.to_path_buf() } else { path.with_extension("tmp") };
    let mut options = std::fs::OpenOptions::new();
    options.write(true).mode(0o600);
    if create_new {
        options.create_new(true);
    } else {
        options.create(true).truncate(true);
    }
    let mut file = options.open(&target).map_err(|e| CliError::Session(format!("{}: {e}", target.display())))?;
    file.write_all(text.as_bytes()).map_err(|e| CliError::Session(e.to_string()))?;
    file.sync_all().map_err(|e| CliError::Session(e.to_string()))?;
    if !create_new {
        std::fs::rename(&target, path).map_err(|e| CliError::Session(format!("{}: {e}", path.display())))?;
    }
    Ok(())
}

/// Založí soubor s klíči uživatele (práva 0600); existující soubor nikdy nepřepíše.
pub fn create_user(keys: &UserKeys, path: &std::path::Path) -> Result<(), CliError> {
    write_private(path, &keys.to_json(), true)
}

/// Uloží klíče (včetně smazání použitých jednorázových prekeys) atomicky: zápis do dočasného souboru a přejmenování.
pub fn save_user(keys: &UserKeys, path: &std::path::Path) -> Result<(), CliError> {
    write_private(path, &keys.to_json(), false)
}

pub fn load_user(path: &std::path::Path) -> Result<UserKeys, CliError> {
    let text = std::fs::read_to_string(path).map_err(|e| CliError::Session(format!("{}: {e}", path.display())))?;
    UserKeys::from_json(&text).ok_or_else(|| CliError::Session(format!("{}: malformed keys", path.display())))
}

/// Výhradní zámek souboru konverzace nebo klíčů. Dva procesy nad stejným stavem ratchetu by jeden druhému přepsaly
/// stav a stejný klíč zprávy by se použil dvakrát; zámek drží, dokud žije vrácený `File`. Zamyká se vedlejší
/// `<soubor>.lock`, protože ukládání soubor nahrazuje přejmenováním (zámek na starém souboru by nic nechránil).
pub fn lock_file(path: &std::path::Path) -> Result<std::fs::File, CliError> {
    use std::os::unix::fs::OpenOptionsExt;
    let mut name = path.as_os_str().to_owned();
    name.push(".lock");
    let lock = std::path::PathBuf::from(name);
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(&lock)
        .map_err(|e| CliError::Session(format!("{}: {e}", lock.display())))?;
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(std::fs::TryLockError::WouldBlock) => {
            Err(CliError::Session(format!("{} is in use by another ancord process (e.g. `online`); write there instead", path.display())))
        }
        Err(std::fs::TryLockError::Error(e)) => Err(CliError::Session(format!("{}: {e}", path.display()))),
    }
}


pub fn load_session(path: &std::path::Path) -> Result<Session, CliError> {
    let text = std::fs::read_to_string(path).map_err(|e| CliError::Session(format!("{}: {e}", path.display())))?;
    serde_json::from_str(&text).map_err(|e| CliError::Session(format!("{}: {e}", path.display())))
}

/// Zapíše konverzaci s právy 0600 (obsahuje klíče).
pub fn save_session(session: &Session, path: &std::path::Path) -> Result<(), CliError> {
    let text = serde_json::to_string_pretty(session).map_err(|e| CliError::Session(e.to_string()))?;
    write_private(path, &text, false)
}
