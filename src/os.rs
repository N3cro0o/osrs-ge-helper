use std::path::PathBuf;
use std::fs::{canonicalize, remove_file};

use crate::{log_mess, log_err};

/// Custom Error struct used to wrap other Error type structs into common one.
#[derive(Debug)]
pub enum OsError {
    IoError(String),
    WinError(String),
    LixError(String),
    Other(String),
}

impl From<std::io::Error> for OsError {
    fn from(error: std::io::Error) -> Self {
        OsError::IoError(error.to_string())
    }
}

impl std::fmt::Display for OsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OsError::IoError(s) => { write!(f, "Input/Output Error - {}", s) }
            OsError::WinError(s) => { write!(f, "Windows related Error - {}", s) }
            OsError::Other(s) => { write!(f, "Other kind of Error - {}", s) }
            OsError::LixError(s) => { write!(f, "Linux related Error - {}", s) }
        }
    } 
}

impl std::error::Error for OsError {}

/// Windows function only. Checks if shell link exists in the autostart directory. Depending on
/// `check` argument, new shell file is created or old one is deleted.
#[cfg(target_os = "windows")]
pub fn toggle_startup_on_boot(check: bool) -> Result<(), OsError> {
    let app_path = std::env::args().next().unwrap();
    let app_path = match canonicalize(PathBuf::from(app_path)) {
        Ok(p) => p,
        Err(err) => { return Err(OsError::from(err)); }
    };
    let mut autostart_path = match dirs::data_dir() {
        Some(path) => path,
        None => { return Err(OsError::IoError("cannot create data_dir path".to_string())); }
    };
    autostart_path.extend(["Microsoft", "Windows", "Start Menu", "Programs", "Startup", "osrs-helper.lnk"]);
    log_mess!["Helper path: {},\nAutostart path {}", app_path.to_str().unwrap(), autostart_path.to_str().unwrap()];
    if check {
        create_shell_link(app_path, autostart_path)
    }
    else {
        remove_shell_link(autostart_path)
    }
}

#[cfg(target_os = "linux")]
pub fn toggle_startup_on_boot(check: bool) -> Result<(), OsError> {
    let app_path = std::env::args().next().unwrap();
    let app_path = match canonicalize(PathBuf::from(app_path)) {
        Ok(p) => p,
        Err(err) => { return Err(OsError::from(err)); }
    };
    let mut autostart_path = match dirs::config_dir() {
        Some(path) => path,
        None => { return Err(OsError::IoError("cannot create data_dir path".to_string())); }
    };
    autostart_path.extend(["autostart", "osrs-helper.desktop"]);
    log_mess!["Helper path: {},\nAutostart path {}", app_path.to_str().unwrap(), autostart_path.to_str().unwrap()];
    if check {
        create_dekstop_file(app_path, autostart_path)
    }
    else {
        remove_shell_link(autostart_path)
    }
}

/// Windows function only. Creates shell link using WinAPI and DOM in the autostart directory. This
/// way of enabling autostart is easier to test and easier for User to manage and check.
#[cfg(target_os = "windows")]
fn create_shell_link(app_path: PathBuf, autostart_path: PathBuf) -> Result<(), OsError> {
    use windows::Win32::System::Com::*;
    use windows::Win32::UI::Shell::*;
    use windows::core::*;

    match autostart_path.try_exists() {
        Ok(b) => { log_mess!["Shell link exists? {}", b]; if b { return Ok(()); } }
        Err(err) => { return Err(OsError::from(err)); }
    }

    unsafe {
        {
            log_mess!["Initializing COM"];
            let result = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            if result.is_err() {
                return Err(OsError::WinError(result.message()));
            }
            let link: IShellLinkW = match CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER) {
                Ok(l) => l,
                Err(err) => {
                    CoUninitialize();
                    return Err(OsError::WinError(err.to_string()));
                }
            };
            let mut s = app_path.display().to_string();
            s = s.replace(r"\\?\", "");
            if let Err(err) = link.SetPath(&HSTRING::from(s)) {
                CoUninitialize();
                return Err(OsError::WinError(err.to_string()));
            };
            if let Err(err) = link.SetDescription(w!("Old school RuneScape Grand Exchange helper.")) {
                CoUninitialize();
                return Err(OsError::WinError(err.to_string()));
            }
            let persist: IPersistFile = link.cast().unwrap();
            if let Err(err) = persist.Save(&HSTRING::from(autostart_path.display().to_string()), true) {
                CoUninitialize();
                return Err(OsError::WinError(err.to_string()));
            };
        }
        CoUninitialize();
        log_mess!["COM done"];
    }
    Ok(())
    // This is certified black magic, damn winapi
}

#[cfg(target_os = "linux")]
fn create_dekstop_file(app_path: PathBuf, autostart_path: PathBuf) -> Result<(), OsError> {
    use std::fs::{OpenOptions, create_dir, Permissions};
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;

    static PERMISSIONS: u32 = 0o744;

    let autostart_path_parent = autostart_path.parent().unwrap();
    println!("{:?}\n{:?}", autostart_path_parent, autostart_path_parent.try_exists());
    if !autostart_path_parent.exists() {
        if let Err(err) = create_dir(autostart_path_parent) {
            return Err(OsError::IoError(err.to_string()));
        }
    }
    match autostart_path.try_exists() {
        Ok(b) => { log_mess!["Shell link exists? {}", b]; if b { return Ok(()); } }
        Err(err) => { return Err(OsError::from(err)); }
    }
    let file_text = format!["[Desktop Entry]\nType=Application\nName={}\nExec={}\nTerminal=false", "OSRS GE Helper", app_path.display().to_string()];
    log_mess![".desktop file: {}", file_text];
    let mut file = match OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .create(true)
        .open(autostart_path) {
            Ok(f) => f,
            Err(err) => { return Err(OsError::LixError(err.to_string())); }
        };
    let mut perms = Permissions::from_mode(PERMISSIONS);
    file.set_permissions(perms);
    match file.write_all(file_text.as_bytes()) {
        Ok(()) => { log_mess![".desktop file done"]; }
        Err(err) => { return Err(OsError::LixError(err.to_string())); }
    };
    Ok(())
}

/// Function used to delete link (for Windows -> shell link) in autostart directory.
fn remove_shell_link(autostart_path: PathBuf) -> Result<(), OsError> {
    let shell_link_exists = match autostart_path.try_exists() {
        Ok(b) => b,
        Err(err) => { return Err(OsError::from(err)); }
    };
    if shell_link_exists {
        if let Err(err) = remove_file(&autostart_path) {
            return Err(OsError::from(err));
        }
    }
    Ok(())
}

/// Abstract function used to check if autostart is enabled.
pub fn check_autostart() -> bool {
    check_target_autostart()
}

/// Windows function only. Used to check if autostart is enabled.
#[cfg(target_os = "windows")]
fn check_target_autostart() -> bool {
    let mut autostart_path = match dirs::data_dir() {
        Some(path) => path,
        None => { log_err!["cannot create data_dir path"]; return false; }
    };
    autostart_path.extend(["Microsoft", "Windows", "Start Menu", "Programs", "Startup", "osrs-helper.lnk"]);
    autostart_path.try_exists().unwrap_or(false)
}

#[cfg(target_os = "linux")]
fn check_target_autostart() -> bool {
    let mut autostart_path = match dirs::config_dir() {
        Some(path) => path,
        None => { log_err!["cannot create config_dir path"]; return false; }
    };
    autostart_path.extend(["autostart", "osrs-helper.desktop"]);
    autostart_path.try_exists().unwrap_or(false)
}
