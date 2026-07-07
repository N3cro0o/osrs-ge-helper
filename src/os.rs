use std::path::PathBuf;
use std::fs::canonicalize;

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
        let string = "XD";
        write!(f, "{}", string)
    } 
}

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
    create_shell_link(app_path, autostart_path);
    Ok(())
}

#[cfg(target_os = "windows")]
fn create_shell_link(app_path: PathBuf, autostart_path: PathBuf) -> Result<(), OsError> {
    use windows::Win32::System::Com::*;
    use windows::Win32::UI::Shell::*;
    use windows::core::*;
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
