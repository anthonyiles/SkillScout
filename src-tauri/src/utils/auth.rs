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
    // Attempt keyring, but don't rely solely on it since it can fail on subsequent reads in some environments
    if let Ok(entry) = keyring::Entry::new("skillscout", "github_token") {
        let _ = entry.set_password(token);
    }

    // Always use the secure file fallback to ensure persistence
    let path = get_token_path(app)?;
    fs::write(&path, token).map_err(|e| format!("Failed to write secure token file: {}", e))?;
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = fs::metadata(&path) {
            let mut perms = meta.permissions();
            perms.set_mode(0o600);
            let _ = fs::set_permissions(&path, perms);
        }
    }
    
    Ok(())
}

pub fn load_token(app: &tauri::AppHandle) -> Result<String, String> {
    if let Ok(entry) = keyring::Entry::new("skillscout", "github_token") {
        if let Ok(token) = entry.get_password() {
            return Ok(token);
        }
    }

    let path = get_token_path(app)?;
    fs::read_to_string(&path).map_err(|e| format!("Token not found: {}", e))
}
