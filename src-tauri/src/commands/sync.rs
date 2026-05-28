use std::process::Command;
use std::path::Path;
use tauri::{Manager, State, Emitter};
use std::fs;
use sha2::{Sha256, Digest};
use hex;

use crate::error::SkillScoutError;
use crate::models::{SyncTask, RepositoryItem, FileHash};
use crate::utils::filesystem::copy_dir_all;
use crate::db::AppState;
use rusqlite::params;

fn canonical_repo_url(url: &str) -> String {
    let trimmed = url.trim().trim_end_matches('/');
    trimmed.strip_suffix(".git").unwrap_or(trimmed).to_string()
}

fn validate_repo_url(url: &str) -> Result<(), SkillScoutError> {
    let url = url.trim();
    let url_normalized = url.strip_suffix(".git").unwrap_or(url);

    let is_https = url_normalized.starts_with("https://github.com/") && {
        let path = &url_normalized["https://github.com/".len()..];
        let path = path.trim_end_matches('/');
        let parts: Vec<&str> = path.splitn(3, '/').collect();
        parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty()
    };

    let is_ssh = url_normalized.starts_with("git@github.com:") && {
        let path = &url_normalized["git@github.com:".len()..];
        let path = path.trim_end_matches('/');
        let parts: Vec<&str> = path.splitn(3, '/').collect();
        parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty()
    };

    if !is_https && !is_ssh {
        return Err(SkillScoutError::RepoUrlInvalid(
            "Repository URL must be a GitHub HTTPS or SSH URL (e.g. https://github.com/org/repo or git@github.com:org/repo)".to_string()
        ));
    }

    let forbidden_chars = ['`', '$', ';', '&', '|', '(', ')', '<', '>', '\n', '\r', '\0'];
    if url.chars().any(|c| forbidden_chars.contains(&c)) {
        return Err(SkillScoutError::RepoUrlInvalid("Repository URL contains invalid characters".to_string()));
    }

    Ok(())
}

#[tauri::command]
pub async fn sync_repo(app: tauri::AppHandle, state: State<'_, AppState>, repo_url: String) -> Result<usize, SkillScoutError> {
    validate_repo_url(&repo_url)?;

    let app_data_dir = app.path().app_data_dir()
        .map_err(|_| SkillScoutError::FileSystemError("Failed to resolve app data directory".to_string()))?;

    let skills = tauri::async_runtime::spawn_blocking(move || {
        if !app_data_dir.exists() {
            fs::create_dir_all(&app_data_dir).map_err(|e| SkillScoutError::FileSystemError(e.to_string()))?;
        }

        let repo_dir = app_data_dir.join("skills_repo");
        let git_dir = repo_dir.join(".git");

        if git_dir.exists() {
            let remote_output = Command::new("git")
                .arg("-C")
                .arg(&repo_dir)
                .arg("remote")
                .arg("get-url")
                .arg("origin")
                .output()
                .map_err(|e| SkillScoutError::GitOperationFailed(e.to_string()))?;

            if !remote_output.status.success() {
                let stderr = String::from_utf8_lossy(&remote_output.stderr);
                return Err(SkillScoutError::GitOperationFailed(format!("Failed to get remote URL: {}", stderr)));
            }

            let current_url = String::from_utf8_lossy(&remote_output.stdout).trim().to_string();

            if canonical_repo_url(&current_url) != canonical_repo_url(&repo_url) {
                fs::remove_dir_all(&repo_dir)
                    .map_err(|e| SkillScoutError::FileSystemError(format!("Failed to clear old repo: {}", e)))?;

                let output = Command::new("git")
                    .arg("clone")
                    .arg(&repo_url)
                    .arg(&repo_dir)
                    .output()
                    .map_err(|e| SkillScoutError::GitOperationFailed(e.to_string()))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(SkillScoutError::GitOperationFailed(format!("Git clone failed: {}", stderr)));
                }
            } else {
                let output = Command::new("git")
                    .arg("-C")
                    .arg(&repo_dir)
                    .arg("pull")
                    .output()
                    .map_err(|e| SkillScoutError::GitOperationFailed(e.to_string()))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(SkillScoutError::GitOperationFailed(format!("Git pull failed: {}", stderr)));
                }
            }
        } else {
            let output = Command::new("git")
                .arg("clone")
                .arg(&repo_url)
                .arg(&repo_dir)
                .output()
                .map_err(|e| SkillScoutError::GitOperationFailed(e.to_string()))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(SkillScoutError::GitOperationFailed(format!("Git clone failed: {}", stderr)));
            }
        }

        let mut skills = Vec::new();

        let mut read_folder = |folder_name: &str| {
            let folder_path = repo_dir.join(folder_name);
            if !folder_path.exists() || !folder_path.is_dir() {
                return;
            }
            let entries = match fs::read_dir(&folder_path) {
                Ok(e) => e,
                Err(e) => { eprintln!("Failed to read {} folder: {}", folder_name, e); return; }
            };
            for entry in entries {
                let entry = match entry {
                    Ok(e) => e,
                    Err(e) => { eprintln!("Failed to read entry in {:?}: {}", folder_path, e); continue; }
                };
                let path = entry.path();
                let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else { continue };
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
                    content = fs::read_to_string(&path).unwrap_or_else(|e| {
                        eprintln!("Failed to read file {:?}: {}", path, e);
                        String::new()
                    });
                } else if path.is_dir() {
                    let skill_md_path = path.join("SKILL.md");
                    if skill_md_path.exists() {
                        content = fs::read_to_string(&skill_md_path).unwrap_or_else(|e| {
                            eprintln!("Failed to read SKILL.md at {:?}: {}", skill_md_path, e);
                            String::new()
                        });
                    } else {
                        match fs::read_dir(&path) {
                            Ok(sub_entries) => {
                                for sub_entry in sub_entries {
                                    let sub_entry = match sub_entry {
                                        Ok(e) => e,
                                        Err(e) => { eprintln!("Failed to read entry in {:?}: {}", path, e); continue; }
                                    };
                                    let sub_path = sub_entry.path();
                                    if sub_path.is_file() && sub_path.extension().and_then(|ext| ext.to_str()) == Some("md") {
                                        content = fs::read_to_string(&sub_path).unwrap_or_else(|e| {
                                            eprintln!("Failed to read {:?}: {}", sub_path, e);
                                            String::new()
                                        });
                                        break;
                                    }
                                }
                            }
                            Err(e) => eprintln!("Failed to read subdirectory {:?}: {}", path, e),
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
                    last_synced: None,
                });
            }
        };
        
        read_folder("skills");
        read_folder("rules");
        
        Ok::<Vec<RepositoryItem>, SkillScoutError>(skills)
    }).await.map_err(|e| {
        eprintln!("Spawn blocking error: {}", e);
        SkillScoutError::GitOperationFailed("Background task failed".to_string())
    })??;

    let count = skills.len();
    let mut conn = state.lock_conn();
    let tx = conn.transaction().map_err(|_| SkillScoutError::DatabaseBusy)?;

    let current_ids: Vec<String> = skills.iter().map(|item| item.id.clone()).collect();
    let placeholders = current_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");

    if !current_ids.is_empty() {
        let query = format!("DELETE FROM repository_items WHERE id NOT IN ({})", placeholders);
        let params: Vec<&dyn rusqlite::ToSql> = current_ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
        tx.execute(&query, rusqlite::params_from_iter(params))
            .map_err(|e| SkillScoutError::DatabaseError(format!("Failed to clean up stale items: {}", e)))?;
    } else {
        tx.execute("DELETE FROM repository_items", [])
            .map_err(|e| SkillScoutError::DatabaseError(e.to_string()))?;
    }

    for skill in &skills {
        tx.execute(
            "INSERT INTO repository_items (id, name, folder, description, file_path, content, sha)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
             name = ?2, folder = ?3, description = ?4, file_path = ?5, content = ?6, sha = ?7, last_synced = CURRENT_TIMESTAMP",
            params![skill.id, skill.name, skill.folder, skill.description, skill.file_path, skill.content, skill.sha]
        ).map_err(|e| SkillScoutError::DatabaseError(format!("Failed to save item: {}", e)))?;

        tx.execute(
            "UPDATE item_selections SET applied_sha = NULL WHERE item_id = ?1 AND applied_sha IS NOT NULL AND applied_sha != ?2",
            params![skill.id, skill.sha]
        ).map_err(|e| SkillScoutError::DatabaseError(format!("Failed to reconcile applied_sha: {}", e)))?;
    }

    tx.commit().map_err(|e| SkillScoutError::DatabaseError(format!("Failed to commit sync: {}", e)))?;

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

fn is_safe_absolute_path(path_str: &str) -> bool {
    let path = Path::new(path_str);
    if !path.is_absolute() {
        return false;
    }
    !path.components().any(|c| matches!(c, std::path::Component::ParentDir))
}

#[tauri::command]
pub fn check_existing(tasks: Vec<SyncTask>) -> Result<Vec<String>, String> {
    let mut existing = Vec::new();
    for task in tasks {
        if !is_safe_absolute_path(&task.target_dir) {
            return Err(SkillScoutError::PathTraversalAttempt.to_string());
        }
        if !is_safe_filename(&task.file_name) {
            return Err(SkillScoutError::PathTraversalAttempt.to_string());
        }
        let target_path = Path::new(&task.target_dir).join(&task.file_name);
        if target_path.exists() {
            existing.push(format!("{}/{}", task.target_dir, task.file_name));
        }
    }
    Ok(existing)
}

#[tauri::command]
pub async fn apply_skills(tasks: Vec<SyncTask>) -> Result<usize, String> {
    let count = tauri::async_runtime::spawn_blocking(move || {
        let mut count = 0;
        for task in tasks {
            if !is_safe_absolute_path(&task.target_dir) {
                return Err(SkillScoutError::PathTraversalAttempt.to_string());
            }
            if !is_safe_filename(&task.file_name) {
                return Err(SkillScoutError::PathTraversalAttempt.to_string());
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
pub fn get_project_file_hashes(project_path: String, sub_folders: Vec<String>) -> Result<Vec<FileHash>, String> {
    let mut hashes: Vec<FileHash> = Vec::new();
    let base_path = Path::new(&project_path);

    for folder in &sub_folders {
        if !is_safe_filename(folder) {
            return Err(SkillScoutError::PathTraversalAttempt.to_string());
        }
        let folder_path = base_path.join(folder);
        if !folder_path.exists() || !folder_path.is_dir() {
            continue;
        }
        let Ok(entries) = fs::read_dir(&folder_path) else { continue };
        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => { eprintln!("Failed to read entry in {:?}: {}", folder_path, e); continue; }
            };
            let path = entry.path();
            let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else { continue };
            if file_name.starts_with('.') {
                continue;
            }
            let content = if path.is_file() {
                fs::read_to_string(&path).unwrap_or_else(|e| {
                    eprintln!("Failed to read {:?}: {}", path, e);
                    String::new()
                })
            } else if path.is_dir() {
                let skill_md = path.join("SKILL.md");
                if skill_md.exists() {
                    fs::read_to_string(&skill_md).unwrap_or_else(|e| {
                        eprintln!("Failed to read SKILL.md at {:?}: {}", skill_md, e);
                        String::new()
                    })
                } else {
                    match fs::read_dir(&path) {
                        Ok(sub) => {
                            let mut found = String::new();
                            for sub_entry in sub {
                                match sub_entry {
                                    Ok(e) if e.path().extension().and_then(|s| s.to_str()) == Some("md") => {
                                        let p = e.path();
                                        found = fs::read_to_string(&p).unwrap_or_else(|err| {
                                            eprintln!("Failed to read {:?}: {}", p, err);
                                            String::new()
                                        });
                                        break;
                                    }
                                    Ok(_) => {}
                                    Err(e) => eprintln!("Failed to read entry in {:?}: {}", path, e),
                                }
                            }
                            found
                        }
                        Err(e) => { eprintln!("Failed to read subdirectory {:?}: {}", path, e); String::new() }
                    }
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

    Ok(hashes)
}

#[tauri::command]
pub fn get_project_files(project_path: String, sub_folders: Vec<String>) -> Result<Vec<String>, String> {
    let mut files = Vec::new();
    let base_path = Path::new(&project_path);

    for folder in sub_folders {
        if !is_safe_filename(&folder) {
            return Err(SkillScoutError::PathTraversalAttempt.to_string());
        }
        let folder_path = base_path.join(&folder);
        if folder_path.exists() && folder_path.is_dir() {
            match fs::read_dir(&folder_path) {
                Ok(entries) => {
                    for entry_res in entries {
                        match entry_res {
                            Ok(entry) => {
                                let path = entry.path();
                                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                                    if !file_name.starts_with('.') {
                                        files.push(file_name.to_string());
                                    }
                                }
                            }
                            Err(e) => eprintln!("Failed to read entry in {:?}: {}", folder_path, e),
                        }
                    }
                }
                Err(e) => eprintln!("Failed to read directory {:?}: {}", folder_path, e),
            }
        }
    }

    files.sort();
    files.dedup();
    Ok(files)
}
