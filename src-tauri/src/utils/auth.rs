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

fn save_token_to_file(_app: &tauri::AppHandle, _token: &str) -> Result<(), String> {
    Err("Secure credential storage is unavailable. Please ensure OS keyring access and try signing in again.".to_string())
}

fn load_token_from_file(_app: &tauri::AppHandle) -> Result<String, String> {
    Err("Not authenticated. Please sign in with GitHub.".to_string())
}
