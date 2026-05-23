use std::fs;
use std::path::PathBuf;
use tauri::Manager;

pub fn get_token_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|_| "Failed to get app data dir".to_string())?;
    if !app_data_dir.exists() {
        fs::create_dir_all(&app_data_dir).map_err(|_| "Failed to create app data dir".to_string())?;
    }
    Ok(app_data_dir.join(".github_token"))
}

pub fn save_token(app: &tauri::AppHandle, token: &str) -> Result<(), String> {
    let keyring_saved = if let Ok(entry) = keyring::Entry::new("skillscout", "github_token") {
        entry.set_password(token).is_ok()
    } else {
        false
    };

    if keyring_saved {
        if let Ok(path) = get_token_path(app) {
            if path.exists() {
                let _ = fs::remove_file(&path);
            }
        }
        return Ok(());
    }

    save_token_to_file(app, token)
}

pub fn load_token(app: &tauri::AppHandle) -> Result<String, String> {
    if let Ok(entry) = keyring::Entry::new("skillscout", "github_token") {
        if let Ok(token) = entry.get_password() {
            return Ok(token);
        }
    }

    load_token_from_file(app)
}

#[cfg(unix)]
fn save_token_to_file(app: &tauri::AppHandle, token: &str) -> Result<(), String> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;

    let path = get_token_path(app)?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .mode(0o600)
        .open(&path)
        .map_err(|e| format!("Failed to write secure token file: {}", e))?;
    file.write_all(token.as_bytes())
        .map_err(|e| format!("Failed to write token: {}", e))?;

    Ok(())
}

#[cfg(not(unix))]
fn save_token_to_file(_app: &tauri::AppHandle, _token: &str) -> Result<(), String> {
    Err("Windows Credential Manager is unavailable. Please ensure it is accessible and try signing in again.".to_string())
}

#[cfg(unix)]
fn load_token_from_file(app: &tauri::AppHandle) -> Result<String, String> {
    let path = get_token_path(app)?;
    fs::read_to_string(&path).map_err(|_| "Not authenticated. Please sign in with GitHub.".to_string())
}

#[cfg(not(unix))]
fn load_token_from_file(_app: &tauri::AppHandle) -> Result<String, String> {
    Err("Not authenticated. Please sign in with GitHub.".to_string())
}
