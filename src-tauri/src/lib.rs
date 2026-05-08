// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::process::Command;
use std::path::{Path, PathBuf};
use tauri::Manager;
use serde::{Serialize, Deserialize};
use std::fs;

const GITHUB_DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
const GITHUB_OAUTH_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const GITHUB_USER_URL: &str = "https://api.github.com/user";

#[derive(Serialize, Deserialize, Clone)]
pub struct Skill {
    id: String,
    name: String,
    folder: String,
    description: String,
    file_path: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
pub struct DeviceAuthResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
}

#[derive(Serialize, Deserialize)]
pub struct TokenResponse {
    access_token: Option<String>,
    token_type: Option<String>,
    scope: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

#[tauri::command]
async fn sync_repo(app: tauri::AppHandle, repo_url: String) -> Result<Vec<Skill>, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    
    if !app_data_dir.exists() {
        fs::create_dir_all(&app_data_dir).map_err(|e| e.to_string())?;
    }
    
    let repo_dir = app_data_dir.join("skills_repo");
    let git_dir = repo_dir.join(".git");
    
    if git_dir.exists() {
        // Check if the URL has changed
        let remote_output = Command::new("git")
            .arg("-C")
            .arg(&repo_dir)
            .arg("remote")
            .arg("get-url")
            .arg("origin")
            .output()
            .map_err(|e| format!("Failed to get remote URL: {}", e))?;
            
        let current_url = String::from_utf8_lossy(&remote_output.stdout).trim().to_string();
        
        if current_url != repo_url.trim() {
            // URL changed, remove the directory and re-clone
            fs::remove_dir_all(&repo_dir).map_err(|e| format!("Failed to clear old repo: {}", e))?;
            
            let output = Command::new("git")
                .arg("clone")
                .arg(&repo_url)
                .arg(&repo_dir)
                .output()
                .map_err(|e| format!("Failed to execute git clone: {}", e))?;
                
            if !output.status.success() {
                let err = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Git clone failed: {}", err));
            }
        } else {
            // Repo exists and URL matches, pull latest changes
            let output = Command::new("git")
                .arg("-C")
                .arg(&repo_dir)
                .arg("pull")
                .output()
                .map_err(|e| format!("Failed to execute git pull: {}", e))?;
                
            if !output.status.success() {
                let err = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Git pull failed: {}", err));
            }
        }
    } else {
        // Repo doesn't exist, clone it
        let output = Command::new("git")
            .arg("clone")
            .arg(&repo_url)
            .arg(&repo_dir)
            .output()
            .map_err(|e| format!("Failed to execute git clone: {}", e))?;
            
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Git clone failed: {}", err));
        }
    }
    
    // Read the skills from the cloned repo
    let mut skills = Vec::new();
    
    // Helper function to read skills from a directory
    let mut read_folder = |folder_name: &str| {
        let folder_path = repo_dir.join(folder_name);
        if folder_path.exists() && folder_path.is_dir() {
            if let Ok(entries) = fs::read_dir(folder_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                        // skip hidden files
                        if file_name.starts_with('.') {
                            continue;
                        }
                        
                        let desc = if path.is_dir() {
                            format!("Directory from {} folder", folder_name)
                        } else {
                            format!("File from {} folder", folder_name)
                        };
                        
                        let mut content = String::new();
                        if path.is_file() {
                            content = fs::read_to_string(&path).unwrap_or_default();
                        } else if path.is_dir() {
                            let skill_md_path = path.join("SKILL.md");
                            if skill_md_path.exists() {
                                content = fs::read_to_string(&skill_md_path).unwrap_or_default();
                            } else {
                                // Try finding any .md file
                                if let Ok(sub_entries) = fs::read_dir(&path) {
                                    for sub_entry in sub_entries.flatten() {
                                        let p = sub_entry.path();
                                        if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("md") {
                                            content = fs::read_to_string(&p).unwrap_or_default();
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        
                        skills.push(Skill {
                            id: format!("{}-{}", folder_name, file_name),
                            name: file_name.to_string(),
                            folder: folder_name.to_string(),
                            description: desc,
                            file_path: path.to_string_lossy().to_string(),
                            content,
                        });
                    }
                }
            }
        }
    };
    
    read_folder("skills");
    read_folder("rules");
    
    Ok(skills)
}

#[derive(Deserialize)]
pub struct SyncTask {
    source_file: Option<String>,
    target_dir: String,
    file_name: String,
    #[serde(default)]
    remove: bool,
}

#[tauri::command]
fn check_existing(tasks: Vec<SyncTask>) -> Vec<String> {
    let mut existing = Vec::new();
    for task in tasks {
        let target_path = Path::new(&task.target_dir).join(&task.file_name);
        if target_path.exists() {
            existing.push(format!("{}/{}", task.target_dir, task.file_name));
        }
    }
    existing
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let path = entry.path();
        let dst_path = dst.as_ref().join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&path, &dst_path)?;
        } else {
            fs::copy(&path, &dst_path)?;
        }
    }
    Ok(())
}

#[tauri::command]
async fn apply_skills(tasks: Vec<SyncTask>) -> Result<usize, String> {
    let count = tauri::async_runtime::spawn_blocking(move || {
        let mut count = 0;
        for task in tasks {
            let target_dir = Path::new(&task.target_dir);
            let target_path = target_dir.join(&task.file_name);
            
            if task.remove {
                // Try removing both file and dir directly without an expensive exists() check
                let _ = fs::remove_file(&target_path);
                let _ = fs::remove_dir_all(&target_path);
            } else {
                if let Some(source_file) = &task.source_file {
                    let source_path = Path::new(source_file);
                    let mut should_copy = true;
                    
                    // Smart copy: skip if file already exists with same size and identical content
                    if target_path.exists() && source_path.is_file() && target_path.is_file() {
                        if let (Ok(src_meta), Ok(tgt_meta)) = (fs::metadata(&source_path), fs::metadata(&target_path)) {
                            if src_meta.len() == tgt_meta.len() {
                                if let (Ok(src_content), Ok(tgt_content)) = (fs::read(&source_path), fs::read(&target_path)) {
                                    if src_content == tgt_content {
                                        should_copy = false;
                                    }
                                }
                            }
                        }
                    }
                    
                    if should_copy {
                        if !target_dir.exists() {
                            fs::create_dir_all(target_dir).map_err(|e| format!("Failed to create directory {:?}: {}", target_dir, e))?;
                        }
                        
                        if source_path.is_dir() {
                            copy_dir_all(&source_path, &target_path).map_err(|e| format!("Failed to copy directory {:?}: {}", target_path, e))?;
                        } else {
                            fs::copy(&source_path, &target_path).map_err(|e| format!("Failed to copy file {:?}: {}", target_path, e))?;
                        }
                        count += 1;
                    }
                }
            }
        }
        Ok::<usize, String>(count)
    }).await.map_err(|e| format!("Task failed to execute: {}", e))??;
    
    Ok(count)
}

#[tauri::command]
fn get_project_files(project_path: String, sub_folders: Vec<String>) -> Vec<String> {
    let mut files = Vec::new();
    let base_path = Path::new(&project_path);
    
    for folder in sub_folders {
        let folder_path = base_path.join(&folder);
        if folder_path.exists() && folder_path.is_dir() {
            if let Ok(entries) = fs::read_dir(folder_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                        if !file_name.starts_with('.') {
                            files.push(file_name.to_string());
                        }
                    }
                }
            }
        }
    }
    
    // Sort and deduplicate
    files.sort();
    files.dedup();
    files
}

fn get_token_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|_| "Failed to get app data dir".to_string())?;
    if !app_data_dir.exists() {
        fs::create_dir_all(&app_data_dir).map_err(|_| "Failed to create app data dir".to_string())?;
    }
    Ok(app_data_dir.join(".github_token"))
}

fn save_token(app: &tauri::AppHandle, token: &str) -> Result<(), String> {
    if let Ok(entry) = keyring::Entry::new("skillscout", "github_token") {
        if entry.set_password(token).is_ok() {
            return Ok(());
        }
    }

    let path = get_token_path(app)?;
    fs::write(&path, token).map_err(|_| "Failed to write secure token file".to_string())?;
    
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

fn load_token(app: &tauri::AppHandle) -> Result<String, String> {
    if let Ok(entry) = keyring::Entry::new("skillscout", "github_token") {
        if let Ok(token) = entry.get_password() {
            return Ok(token);
        }
    }

    let path = get_token_path(app)?;
    fs::read_to_string(&path).map_err(|_| "Token not found".to_string())
}

#[tauri::command]
async fn logout_github(app: tauri::AppHandle) -> Result<(), String> {
    if let Ok(entry) = keyring::Entry::new("skillscout", "github_token") {
        let _ = entry.delete_credential();
    }

    if let Ok(path) = get_token_path(&app) {
        let _ = fs::remove_file(path);
    }
    Ok(())
}

#[tauri::command]
async fn check_github_auth(app: tauri::AppHandle) -> Result<bool, String> {
    let token = match load_token(&app) {
        Ok(t) => t,
        Err(_) => return Ok(false),
    };

    let client = reqwest::Client::new();
    let res = client.get(GITHUB_USER_URL)
        .header("User-Agent", "SkillScout-App")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| "Network error while verifying GitHub connection.".to_string())?;

    Ok(res.status().is_success())
}

#[tauri::command]
async fn start_github_device_flow() -> Result<DeviceAuthResponse, String> {
    let client_id = std::env::var("VITE_GITHUB_CLIENT_ID").unwrap_or_default();
    if client_id.is_empty() || client_id == "your_sandbox_client_id_here" {
        return Err("GitHub Client ID is missing or invalid in configuration.".to_string());
    }

    let client = reqwest::Client::new();
    let res = client.post(GITHUB_DEVICE_CODE_URL)
        .header("Accept", "application/json")
        .query(&[
            ("client_id", &client_id),
            ("scope", &"repo".to_string())
        ])
        .send()
        .await
        .map_err(|_| "Network error while starting authentication flow.".to_string())?;

    let auth_res = res.json::<DeviceAuthResponse>().await.map_err(|_| "Failed to process GitHub's response.".to_string())?;
    Ok(auth_res)
}

#[tauri::command]
async fn poll_github_token(app: tauri::AppHandle, device_code: String) -> Result<TokenResponse, String> {
    let client_id = std::env::var("VITE_GITHUB_CLIENT_ID").unwrap_or_default();

    let client = reqwest::Client::new();
    let res = client.post(GITHUB_OAUTH_TOKEN_URL)
        .header("Accept", "application/json")
        .query(&[
            ("client_id", &client_id),
            ("device_code", &device_code),
            ("grant_type", &"urn:ietf:params:oauth:grant-type:device_code".to_string()),
        ])
        .send()
        .await
        .map_err(|_| "Network error while polling for access token.".to_string())?;

    let token_res = res.json::<TokenResponse>().await.map_err(|_| "Failed to process GitHub's response.".to_string())?;
    
    // Securely save the token if it was returned
    if let Some(token) = &token_res.access_token {
        save_token(&app, token)?;
    }
    
    Ok(token_res)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenvy::dotenv().ok(); // Load environment variables from .env

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            sync_repo, check_existing, apply_skills, get_project_files, 
            start_github_device_flow, poll_github_token, check_github_auth, logout_github
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
