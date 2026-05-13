use std::fs;
use std::path::Path;
use base64::{Engine as _, engine::general_purpose};

pub fn parse_repo_url(url: &str) -> Option<(String, String)> {
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

pub async fn get_repo_info(client: &reqwest::Client, api_base: &str, token: &str) -> Result<(String, String), String> {
    let repo_res = client.get(api_base)
        .header("User-Agent", "SkillScout-App")
        .header("Authorization", format!("Bearer {}", token))
        .send().await
        .map_err(|_| "Failed to connect to GitHub.".to_string())?;

    if !repo_res.status().is_success() {
        return Err(format!("Failed to fetch repository info: {}", repo_res.status()));
    }
    let repo_info: serde_json::Value = repo_res.json().await
        .map_err(|_| "Failed to parse repository info from GitHub.".to_string())?;
    let default_branch = repo_info["default_branch"].as_str().unwrap_or("main").to_string();

    let ref_url = format!("{}/git/refs/heads/{}", api_base, default_branch);
    let ref_res = client.get(&ref_url)
        .header("User-Agent", "SkillScout-App")
        .header("Authorization", format!("Bearer {}", token))
        .send().await
        .map_err(|_| "Failed to connect to GitHub.".to_string())?;

    if !ref_res.status().is_success() {
        return Err(format!("Failed to fetch branch reference: {}", ref_res.status()));
    }
    let ref_info: serde_json::Value = ref_res.json().await
        .map_err(|_| "Failed to parse branch reference from GitHub.".to_string())?;
    let latest_sha = ref_info["object"]["sha"]
        .as_str()
        .ok_or("Could not read latest commit SHA from GitHub.")?
        .to_string();

    Ok((default_branch, latest_sha))
}

pub async fn create_branch(client: &reqwest::Client, api_base: &str, token: &str, branch_name: &str, sha: &str) -> Result<(), String> {
    let branch_url = format!("{}/git/refs", api_base);
    let res = client.post(&branch_url)
        .header("User-Agent", "SkillScout-App")
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "ref": format!("refs/heads/{}", branch_name),
            "sha": sha
        }))
        .send().await
        .map_err(|_| "Failed to connect to GitHub.".to_string())?;

    if !res.status().is_success() {
        let err_text = res.text().await.unwrap_or_default();
        return Err(format!("Failed to create branch: {}", err_text));
    }
    Ok(())
}

struct BlobEntry {
    repo_path: String,
    content: Vec<u8>,
}

pub async fn create_blobs_for_item(
    client: &reqwest::Client,
    api_base: &str,
    token: &str,
    item_path: &Path,
    item_name: &str,
    item_type: &str,
) -> Result<Vec<serde_json::Value>, String> {
    let is_dir = item_path.is_dir();
    let target_root = if item_type == "skill" { "skills" } else { "rules" };

    // Collect all file paths and their repo-relative destinations synchronously
    let mut entries: Vec<BlobEntry> = Vec::new();
    let mut paths_to_process = vec![item_path.to_path_buf()];
    while let Some(current_path) = paths_to_process.pop() {
        if current_path.is_dir() {
            if let Ok(dir_entries) = fs::read_dir(&current_path) {
                for entry in dir_entries.flatten() {
                    let p = entry.path();
                    if p.file_name().and_then(|n| n.to_str()).map(|s| s.starts_with('.')).unwrap_or(false) {
                        continue;
                    }
                    paths_to_process.push(p);
                }
            }
        } else if current_path.is_file() {
            let content = fs::read(&current_path).map_err(|e| format!("Failed to read file {:?}: {}", current_path, e))?;
            let rel_path = current_path.strip_prefix(item_path).unwrap_or(&current_path);
            let repo_path = if is_dir {
                format!("{}/{}/{}", target_root, item_name, rel_path.to_string_lossy()).replace('\\', "/")
            } else {
                format!("{}/{}", target_root, item_name)
            };
            entries.push(BlobEntry { repo_path, content });
        }
    }

    // Upload all blobs concurrently
    let blob_url = format!("{}/git/blobs", api_base);
    let mut handles = Vec::new();
    for entry in entries {
        let client = client.clone();
        let blob_url = blob_url.clone();
        let token = token.to_string();
        let encoded = general_purpose::STANDARD.encode(&entry.content);
        let repo_path = entry.repo_path.clone();
        handles.push(tauri::async_runtime::spawn(async move {
            let res = client.post(&blob_url)
                .header("User-Agent", "SkillScout-App")
                .header("Authorization", format!("Bearer {}", token))
                .json(&serde_json::json!({ "content": encoded, "encoding": "base64" }))
                .send().await
                .map_err(|_| "Failed to connect to GitHub while uploading file.".to_string())?;

            if !res.status().is_success() {
                return Err(format!("Failed to upload file to GitHub: {}", res.status()));
            }
            let info: serde_json::Value = res.json().await
                .map_err(|_| "Failed to parse blob response from GitHub.".to_string())?;
            let sha = info["sha"].as_str().ok_or("Missing SHA in blob response.")?.to_string();
            Ok(serde_json::json!({
                "path": repo_path,
                "mode": "100644",
                "type": "blob",
                "sha": sha
            }))
        }));
    }

    let mut tree_entries = Vec::new();
    for handle in handles {
        tree_entries.push(handle.await.map_err(|_| "File upload task panicked.".to_string())??);
    }
    Ok(tree_entries)
}

pub async fn create_tree(client: &reqwest::Client, api_base: &str, token: &str, base_tree_sha: &str, tree_entries: Vec<serde_json::Value>) -> Result<String, String> {
    let tree_url = format!("{}/git/trees", api_base);
    let res = client.post(&tree_url)
        .header("User-Agent", "SkillScout-App")
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({ "base_tree": base_tree_sha, "tree": tree_entries }))
        .send().await
        .map_err(|_| "Failed to connect to GitHub.".to_string())?;

    if !res.status().is_success() {
        return Err(format!("Failed to create tree on GitHub: {}", res.status()));
    }
    let info: serde_json::Value = res.json().await
        .map_err(|_| "Failed to parse tree response from GitHub.".to_string())?;
    Ok(info["sha"].as_str().ok_or("Missing SHA in tree response.")?.to_string())
}

pub async fn create_commit(client: &reqwest::Client, api_base: &str, token: &str, message: &str, tree_sha: &str, parent_sha: &str) -> Result<String, String> {
    let commit_url = format!("{}/git/commits", api_base);
    let res = client.post(&commit_url)
        .header("User-Agent", "SkillScout-App")
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({ "message": message, "tree": tree_sha, "parents": [parent_sha] }))
        .send().await
        .map_err(|_| "Failed to connect to GitHub.".to_string())?;

    if !res.status().is_success() {
        return Err(format!("Failed to create commit on GitHub: {}", res.status()));
    }
    let info: serde_json::Value = res.json().await
        .map_err(|_| "Failed to parse commit response from GitHub.".to_string())?;
    Ok(info["sha"].as_str().ok_or("Missing SHA in commit response.")?.to_string())
}

pub async fn update_branch(client: &reqwest::Client, api_base: &str, token: &str, branch_name: &str, commit_sha: &str) -> Result<(), String> {
    let update_url = format!("{}/git/refs/heads/{}", api_base, branch_name);
    let res = client.patch(&update_url)
        .header("User-Agent", "SkillScout-App")
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({ "sha": commit_sha, "force": true }))
        .send().await
        .map_err(|_| "Failed to connect to GitHub.".to_string())?;

    if !res.status().is_success() {
        return Err(format!("Failed to update branch on GitHub: {}", res.status()));
    }
    Ok(())
}

pub async fn create_pull_request(client: &reqwest::Client, api_base: &str, token: &str, title: &str, head: &str, base: &str, body: &str) -> Result<String, String> {
    let pulls_url = format!("{}/pulls", api_base);
    let res = client.post(&pulls_url)
        .header("User-Agent", "SkillScout-App")
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({ "title": title, "head": head, "base": base, "body": body }))
        .send().await
        .map_err(|_| "Failed to connect to GitHub.".to_string())?;

    if !res.status().is_success() {
        let err_text = res.text().await.unwrap_or_default();
        return Err(format!("Failed to create pull request: {}", err_text));
    }
    let info: serde_json::Value = res.json().await
        .map_err(|_| "Failed to parse pull request response from GitHub.".to_string())?;
    Ok(info["html_url"].as_str().ok_or("Missing URL in pull request response.")?.to_string())
}

pub async fn delete_branch(client: &reqwest::Client, api_base: &str, token: &str, branch_name: &str) -> Result<(), String> {
    let delete_url = format!("{}/git/refs/heads/{}", api_base, branch_name);
    let res = client.delete(&delete_url)
        .header("User-Agent", "SkillScout-App")
        .header("Authorization", format!("Bearer {}", token))
        .send().await
        .map_err(|_| "Failed to connect to GitHub.".to_string())?;

    if !res.status().is_success() {
        return Err(format!("Failed to delete branch on GitHub: {}", res.status()));
    }
    Ok(())
}
