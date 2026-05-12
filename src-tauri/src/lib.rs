// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::process::Command;
use std::path::{Path, PathBuf};
use tauri::Manager;
use serde::{Serialize, Deserialize};
use std::fs;
use base64::{Engine as _, engine::general_purpose};

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
    #[serde(default)]
    device_code: String,
    #[serde(default)]
    user_code: String,
    #[serde(default)]
    verification_uri: String,
    #[serde(default)]
    expires_in: u64,
    #[serde(default)]
    interval: u64,
    error: Option<String>,
    error_description: Option<String>,
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

fn load_token(app: &tauri::AppHandle) -> Result<String, String> {
    if let Ok(entry) = keyring::Entry::new("skillscout", "github_token") {
        if let Ok(token) = entry.get_password() {
            return Ok(token);
        }
    }

    let path = get_token_path(app)?;
    fs::read_to_string(&path).map_err(|e| format!("Token not found: {}", e))
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

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|_| "Failed to build client".to_string())?;
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
    let client_id = std::env::var("GITHUB_CLIENT_ID").unwrap_or_default();
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

    let status = res.status();
    let body_text = res.text().await.map_err(|_| "Failed to read response body.".to_string())?;

    if !status.is_success() {
        if let Ok(err_json) = serde_json::from_str::<serde_json::Value>(&body_text) {
            if let Some(err_desc) = err_json.get("error_description").and_then(|v| v.as_str()) {
                return Err(err_desc.to_string());
            } else if let Some(err) = err_json.get("error").and_then(|v| v.as_str()) {
                return Err(err.to_string());
            }
        }
        return Err(format!("GitHub API error: {}", status));
    }

    let auth_res = serde_json::from_str::<DeviceAuthResponse>(&body_text).map_err(|_| "Failed to process GitHub's response.".to_string())?;
    
    if auth_res.error.is_some() {
        return Err(auth_res.error_description.clone().unwrap_or_else(|| auth_res.error.clone().unwrap()));
    }
    
    Ok(auth_res)
}

#[tauri::command]
async fn poll_github_token(app: tauri::AppHandle, device_code: String) -> Result<TokenResponse, String> {
    let client_id = std::env::var("GITHUB_CLIENT_ID").unwrap_or_default();
    if client_id.is_empty() || client_id == "your_sandbox_client_id_here" {
        return Err("GitHub Client ID is missing or invalid in configuration.".to_string());
    }

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

#[tauri::command]
async fn check_pr_status(app: tauri::AppHandle, pr_url: String) -> Result<serde_json::Value, String> {
    let token = load_token(&app)?;
    
    let parts: Vec<&str> = pr_url.split("github.com/").collect();
    if parts.len() < 2 {
        return Err("Invalid PR URL".into());
    }
    let path_parts: Vec<&str> = parts[1].split('/').collect();
    if path_parts.len() < 4 || path_parts[2] != "pull" {
        return Err("Invalid PR URL format".into());
    }
    
    let owner = path_parts[0];
    let repo = path_parts[1];
    let pull_number = path_parts[3];

    let api_url = format!("https://api.github.com/repos/{}/{}/pulls/{}", owner, repo, pull_number);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to build client: {}", e))?;

    let res = client.get(&api_url)
        .header("User-Agent", "SkillScout-App")
        .header("Authorization", format!("Bearer {}", token))
        .send().await.map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("Failed to fetch PR status: {}", res.status()));
    }

    let pr_info: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    let state = pr_info["state"].as_str().unwrap_or("unknown").to_string();
    let merged = pr_info["merged"].as_bool().unwrap_or(false);
    
    Ok(serde_json::json!({
        "state": state,
        "merged": merged
    }))
}

fn parse_repo_url(url: &str) -> Option<(String, String)> {
    let url = url.trim_end_matches(".git").trim();
    
    if url.starts_with("git@github.com:") {
        let path = url.trim_start_matches("git@github.com:");
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() == 2 {
            return Some((parts[0].to_string(), parts[1].to_string()));
        }
    } else if url.contains("github.com/") {
        let parts: Vec<&str> = url.split("github.com/").collect();
        if parts.len() > 1 {
            let repo_parts: Vec<&str> = parts[1].split('/').collect();
            if repo_parts.len() >= 2 {
                return Some((repo_parts[0].to_string(), repo_parts[1].to_string()));
            }
        }
    } else {
        let parts: Vec<&str> = url.split('/').collect();
        if parts.len() == 2 && !url.contains("://") && !url.contains('@') {
            return Some((parts[0].to_string(), parts[1].to_string()));
        }
    }
    None
}

async fn get_repo_info(client: &reqwest::Client, api_base: &str, token: &str) -> Result<(String, String), String> {
    let repo_res = client.get(api_base)
        .header("User-Agent", "SkillScout-App")
        .header("Authorization", format!("Bearer {}", token))
        .send().await.map_err(|e| e.to_string())?;
    
    if !repo_res.status().is_success() {
        return Err(format!("Failed to fetch repo info: {}", repo_res.status()));
    }
    let repo_info: serde_json::Value = repo_res.json().await.map_err(|e| e.to_string())?;
    let default_branch = repo_info["default_branch"].as_str().unwrap_or("main").to_string();

    let ref_url = format!("{}/git/refs/heads/{}", api_base, default_branch);
    let ref_res = client.get(&ref_url)
        .header("User-Agent", "SkillScout-App")
        .header("Authorization", format!("Bearer {}", token))
        .send().await.map_err(|e| e.to_string())?;
        
    if !ref_res.status().is_success() {
        return Err(format!("Failed to fetch default branch ref: {}", ref_res.status()));
    }
    let ref_info: serde_json::Value = ref_res.json().await.map_err(|e| e.to_string())?;
    let latest_sha = ref_info["object"]["sha"].as_str().ok_or("Failed to get SHA")?.to_string();

    Ok((default_branch, latest_sha))
}

async fn create_branch(client: &reqwest::Client, api_base: &str, token: &str, branch_name: &str, sha: &str) -> Result<(), String> {
    let branch_url = format!("{}/git/refs", api_base);
    let create_branch_res = client.post(&branch_url)
        .header("User-Agent", "SkillScout-App")
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "ref": format!("refs/heads/{}", branch_name),
            "sha": sha
        }))
        .send().await.map_err(|e| e.to_string())?;

    if !create_branch_res.status().is_success() {
        let err_text = create_branch_res.text().await.unwrap_or_default();
        return Err(format!("Failed to create branch: {}", err_text));
    }
    Ok(())
}

async fn create_blobs_for_item(client: &reqwest::Client, api_base: &str, token: &str, item_path: &Path, item_name: &str, item_type: &str) -> Result<Vec<serde_json::Value>, String> {
    let mut tree_entries = Vec::new();
    let is_dir = item_path.is_dir();
    let mut paths_to_process = vec![item_path.to_path_buf()];
    
    while let Some(current_path) = paths_to_process.pop() {
        if current_path.is_dir() {
            if let Ok(entries) = fs::read_dir(&current_path) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.file_name().and_then(|n| n.to_str()).map(|s| s.starts_with('.')).unwrap_or(false) {
                        continue;
                    }
                    paths_to_process.push(p);
                }
            }
        } else if current_path.is_file() {
            let content = fs::read(&current_path).map_err(|e| e.to_string())?;
            let blob_content = general_purpose::STANDARD.encode(&content);
            
            let blob_url = format!("{}/git/blobs", api_base);
            let blob_res = client.post(&blob_url)
                .header("User-Agent", "SkillScout-App")
                .header("Authorization", format!("Bearer {}", token))
                .json(&serde_json::json!({
                    "content": blob_content,
                    "encoding": "base64"
                }))
                .send().await.map_err(|e| e.to_string())?;
                
            if !blob_res.status().is_success() {
                return Err(format!("Failed to create blob for {:?}: {}", current_path, blob_res.status()));
            }
            let blob_info: serde_json::Value = blob_res.json().await.map_err(|e| e.to_string())?;
            let blob_sha = blob_info["sha"].as_str().ok_or("Failed to get blob SHA")?;
            
            let target_root = if item_type == "skill" { "skills" } else { "rules" };
            let rel_path = current_path.strip_prefix(item_path).unwrap_or(&current_path);
            let repo_path = if is_dir {
                format!("{}/{}/{}", target_root, item_name, rel_path.to_string_lossy())
            } else {
                format!("{}/{}", target_root, item_name)
            };
            
            tree_entries.push(serde_json::json!({
                "path": repo_path.replace("\\", "/"),
                "mode": "100644",
                "type": "blob",
                "sha": blob_sha
            }));
        }
    }
    Ok(tree_entries)
}

async fn create_tree(client: &reqwest::Client, api_base: &str, token: &str, base_tree_sha: &str, tree_entries: Vec<serde_json::Value>) -> Result<String, String> {
    let tree_url = format!("{}/git/trees", api_base);
    let tree_res = client.post(&tree_url)
        .header("User-Agent", "SkillScout-App")
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "base_tree": base_tree_sha,
            "tree": tree_entries
        }))
        .send().await.map_err(|e| e.to_string())?;
        
    if !tree_res.status().is_success() {
        return Err(format!("Failed to create tree: {}", tree_res.status()));
    }
    let tree_info: serde_json::Value = tree_res.json().await.map_err(|e| e.to_string())?;
    Ok(tree_info["sha"].as_str().ok_or("Failed to get new tree SHA")?.to_string())
}

async fn create_commit(client: &reqwest::Client, api_base: &str, token: &str, message: &str, tree_sha: &str, parent_sha: &str) -> Result<String, String> {
    let commit_url = format!("{}/git/commits", api_base);
    let commit_res = client.post(&commit_url)
        .header("User-Agent", "SkillScout-App")
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "message": message,
            "tree": tree_sha,
            "parents": [parent_sha]
        }))
        .send().await.map_err(|e| e.to_string())?;
        
    if !commit_res.status().is_success() {
        return Err(format!("Failed to create commit: {}", commit_res.status()));
    }
    let commit_info: serde_json::Value = commit_res.json().await.map_err(|e| e.to_string())?;
    Ok(commit_info["sha"].as_str().ok_or("Failed to get new commit SHA")?.to_string())
}

async fn update_branch(client: &reqwest::Client, api_base: &str, token: &str, branch_name: &str, commit_sha: &str) -> Result<(), String> {
    let update_ref_url = format!("{}/git/refs/heads/{}", api_base, branch_name);
    let update_res = client.patch(&update_ref_url)
        .header("User-Agent", "SkillScout-App")
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "sha": commit_sha,
            "force": true
        }))
        .send().await.map_err(|e| e.to_string())?;
        
    if !update_res.status().is_success() {
        return Err(format!("Failed to update branch: {}", update_res.status()));
    }
    Ok(())
}

async fn create_pull_request(client: &reqwest::Client, api_base: &str, token: &str, title: &str, head: &str, base: &str, body: &str) -> Result<String, String> {
    let pulls_url = format!("{}/pulls", api_base);
    let pr_res = client.post(&pulls_url)
        .header("User-Agent", "SkillScout-App")
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "title": title,
            "head": head,
            "base": base,
            "body": body
        }))
        .send().await.map_err(|e| e.to_string())?;
        
    if !pr_res.status().is_success() {
        let err_text = pr_res.text().await.unwrap_or_default();
        return Err(format!("Failed to create PR: {}", err_text));
    }
    
    let pr_info: serde_json::Value = pr_res.json().await.map_err(|e| e.to_string())?;
    Ok(pr_info["html_url"].as_str().ok_or("Failed to get PR URL")?.to_string())
}

#[tauri::command]
async fn promote_item(
    app: tauri::AppHandle,
    repo_url: String,
    item_type: String,
    item_name: String,
    project_path: String,
    sub_folders: Vec<String>,
) -> Result<String, String> {
    let token = load_token(&app)?;
    let (owner, repo) = parse_repo_url(&repo_url).ok_or("Invalid repository URL format.")?;

    let base_path = Path::new(&project_path);
    let mut item_path = None;
    for folder in sub_folders {
        let path = base_path.join(&folder).join(&item_name);
        if path.exists() {
            item_path = Some(path);
            break;
        }
    }
    let item_path = item_path.ok_or("Could not locate the item locally.")?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to build client: {}", e))?;

    let api_base = format!("https://api.github.com/repos/{}/{}", owner, repo);

    let (default_branch, latest_sha) = get_repo_info(&client, &api_base, &token).await?;

    let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
    let safe_name = item_name.replace(|c: char| !c.is_alphanumeric(), "-");
    let new_branch_name = format!("promote-{}-{}", safe_name, timestamp);
    create_branch(&client, &api_base, &token, &new_branch_name, &latest_sha).await?;

    let tree_entries = create_blobs_for_item(&client, &api_base, &token, &item_path, &item_name, &item_type).await?;

    let new_tree_sha = create_tree(&client, &api_base, &token, &latest_sha, tree_entries).await?;

    let commit_message = format!("Promote {}: {}", item_type, item_name);
    let new_commit_sha = create_commit(&client, &api_base, &token, &commit_message, &new_tree_sha, &latest_sha).await?;

    update_branch(&client, &api_base, &token, &new_branch_name, &new_commit_sha).await?;

    let pr_body = format!("Automated PR to promote the {} `{}` from local environment.", item_type, item_name);
    let html_url = create_pull_request(&client, &api_base, &token, &commit_message, &new_branch_name, &default_branch, &pr_body).await?;
    
    Ok(html_url)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenvy::dotenv().ok(); // Load environment variables from .env

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            sync_repo, check_existing, apply_skills, get_project_files, 
            start_github_device_flow, poll_github_token, check_github_auth, logout_github, promote_item, check_pr_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
