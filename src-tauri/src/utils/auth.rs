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
                fs::remove_file(&path)
                    .map_err(|e| format!("Failed to remove token fallback file: {}", e))?;
            }
        }
        return Ok(());
    }

    {
        let path = get_token_path(app)?;
        fs::write(&path, token).map_err(|e| format!("Failed to write secure token file: {}", e))?;
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let meta = fs::metadata(&path).map_err(|e| format!("Failed to stat token file: {}", e))?;
            let mut perms = meta.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&path, perms)
                .map_err(|e| format!("Failed to harden token file permissions: {}", e))?;
        }
    }
    
    Ok(())
}
        let path = get_token_path(app)?;
        fs::write(&path, token).map_err(|e| format!("Failed to write secure token file: {}", e))?;
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let meta = fs::metadata(&path).map_err(|e| format!("Failed to stat token file: {}", e))?;
            let mut perms = meta.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&path, perms)
                .map_err(|e| format!("Failed to harden token file permissions: {}", e))?;
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
