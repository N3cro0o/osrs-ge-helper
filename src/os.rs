use std::path::PathBuf;
use std::fs::{canonicalize, remove_file};

use crate::{log_mess, log_err};

#[derive(Debug)]
pub enum OsError {
    IoError(String),
    WinError(String),
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
        }
    } 
}

impl std::error::Error for OsError {}

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
}

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

pub fn check_autostart() -> bool {
    check_target_autostart()
}

#[cfg(target_os = "windows")]
fn check_target_autostart() -> bool {
    let mut autostart_path = match dirs::data_dir() {
        Some(path) => path,
        None => { log_err!["cannot create data_dir path"]; return false; }
    };
    autostart_path.extend(["Microsoft", "Windows", "Start Menu", "Programs", "Startup", "osrs-helper.lnk"]);
    autostart_path.try_exists().unwrap_or(false)
}
