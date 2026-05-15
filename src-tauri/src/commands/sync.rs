use std::process::Command;
use std::path::Path;
use tauri::{Manager, State, Emitter};
use std::fs;
use sha2::{Sha256, Digest};
use hex;

use crate::models::{SyncTask, RepositoryItem, FileHash};
use crate::utils::filesystem::copy_dir_all;
use crate::db::AppState;
use rusqlite::params;

#[tauri::command]
pub async fn sync_repo(app: tauri::AppHandle, state: State<'_, AppState>, repo_url: String) -> Result<usize, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| { eprintln!("App data dir error: {}", e); "Failed to resolve app data directory".to_string() })?;
    
    let skills = tauri::async_runtime::spawn_blocking(move || {
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
                
            if !remote_output.status.success() {
                let err = String::from_utf8_lossy(&remote_output.stderr);
                return Err(format!("Failed to get remote URL: {}", err));
            }
            
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
                            
                            let mut hasher = Sha256::new();
                            hasher.update(&content);
                            let sha = hex::encode(hasher.finalize());

                            skills.push(RepositoryItem {
                                id: format!("{}-{}", folder_name, file_name),
                                name: file_name.to_string(),
                                folder: folder_name.to_string(),
                                description: Some(desc),
                                file_path: path.to_string_lossy().to_string(),
                                content,
                                sha,
                                last_synced: None, // SQLite handles DEFAULT CURRENT_TIMESTAMP on insert
                            });
                        }
                    }
                }
            }
        };
        
        read_folder("skills");
        read_folder("rules");
        
        Ok::<Vec<RepositoryItem>, String>(skills)
    }).await.map_err(|e| { eprintln!("Spawn blocking error: {}", e); "Background task failed".to_string() })??;
    
    // Save to DB
    let count = skills.len();
    let mut conn = state.db.lock().map_err(|e| { eprintln!("Database lock error: {}", e); "Database busy".to_string() })?;
    let tx = conn.transaction().map_err(|e| { eprintln!("Database transaction error: {}", e); "Database busy".to_string() })?;
    
    // Clear old items not in the newly synced list (or just overwrite)
    let current_ids: Vec<String> = skills.iter().map(|s| s.id.clone()).collect();
    
    // Create placeholders for NOT IN query
    let placeholders = current_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    
    if !current_ids.is_empty() {
        let query = format!("DELETE FROM repository_items WHERE id NOT IN ({})", placeholders);
        let params: Vec<&dyn rusqlite::ToSql> = current_ids.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
        tx.execute(&query, rusqlite::params_from_iter(params)).map_err(|e| { eprintln!("Database execute error: {}", e); "Failed to cleanup repository items".to_string() })?;
    } else {
        tx.execute("DELETE FROM repository_items", []).map_err(|e| { eprintln!("Database execute error: {}", e); "Failed to cleanup repository items".to_string() })?;
    }

    for skill in skills {
        tx.execute(
            "INSERT INTO repository_items (id, name, folder, description, file_path, content, sha) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET 
             name = ?2, folder = ?3, description = ?4, file_path = ?5, content = ?6, sha = ?7, last_synced = CURRENT_TIMESTAMP",
            params![skill.id, skill.name, skill.folder, skill.description, skill.file_path, skill.content, skill.sha]
        ).map_err(|e| { eprintln!("Database execute error: {}", e); "Failed to save repository item".to_string() })?;
    }
    
    tx.commit().map_err(|e| { eprintln!("Database commit error: {}", e); "Failed to save changes".to_string() })?;

    // Inform the frontend of a successful sync
    let _ = app.emit("repo_synced", ());

    Ok(count)
}

fn is_safe_filename(name: &str) -> bool {
    let path = Path::new(name);
    if path.is_absolute() {
        return false;
    }
    !path.components().any(|c| matches!(c, std::path::Component::ParentDir | std::path::Component::RootDir | std::path::Component::Prefix(_)))
}

#[tauri::command]
pub fn check_existing(tasks: Vec<SyncTask>) -> Vec<String> {
    let mut existing = Vec::new();
    for task in tasks {
        if !is_safe_filename(&task.file_name) {
            continue;
        }
        let target_path = Path::new(&task.target_dir).join(&task.file_name);
        if target_path.exists() {
            existing.push(format!("{}/{}", task.target_dir, task.file_name));
        }
    }
    existing
}

#[tauri::command]
pub async fn apply_skills(tasks: Vec<SyncTask>) -> Result<usize, String> {
    let count = tauri::async_runtime::spawn_blocking(move || {
        let mut count = 0;
        for task in tasks {
            if !is_safe_filename(&task.file_name) {
                continue;
            }
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
    }).await.map_err(|e| { eprintln!("Spawn blocking error: {}", e); "Background task failed".to_string() })??;
    
    Ok(count)
}

#[tauri::command]
pub fn get_project_file_hashes(project_path: String, sub_folders: Vec<String>) -> Vec<FileHash> {
    let mut hashes: Vec<FileHash> = Vec::new();
    let base_path = Path::new(&project_path);

    for folder in &sub_folders {
        if !is_safe_filename(folder) {
            continue;
        }
        let folder_path = base_path.join(folder);
        if !folder_path.exists() || !folder_path.is_dir() {
            continue;
        }
        let Ok(entries) = fs::read_dir(folder_path) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else { continue };
            if file_name.starts_with('.') {
                continue;
            }
            let content = if path.is_file() {
                fs::read_to_string(&path).unwrap_or_default()
            } else if path.is_dir() {
                let skill_md = path.join("SKILL.md");
                if skill_md.exists() {
                    fs::read_to_string(&skill_md).unwrap_or_default()
                } else if let Ok(sub) = fs::read_dir(&path) {
                    sub.flatten()
                        .find(|e| e.path().extension().and_then(|s| s.to_str()) == Some("md"))
                        .and_then(|e| fs::read_to_string(e.path()).ok())
                        .unwrap_or_default()
                } else {
                    String::new()
                }
            } else {
                continue;
            };
            let mut hasher = Sha256::new();
            hasher.update(&content);
            hashes.push(FileHash {
                name: file_name.to_string(),
                sha: hex::encode(hasher.finalize()),
                folder: folder.clone(),
                content,
            });
        }
    }

    hashes
}

#[tauri::command]
pub fn get_project_files(project_path: String, sub_folders: Vec<String>) -> Vec<String> {
    let mut files = Vec::new();
    let base_path = Path::new(&project_path);

    for folder in sub_folders {
        if !is_safe_filename(&folder) {
            continue;
        }
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

    files.sort();
    files.dedup();
    files
}
