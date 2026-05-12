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

pub async fn create_branch(client: &reqwest::Client, api_base: &str, token: &str, branch_name: &str, sha: &str) -> Result<(), String> {
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

pub async fn create_blobs_for_item(client: &reqwest::Client, api_base: &str, token: &str, item_path: &Path, item_name: &str, item_type: &str) -> Result<Vec<serde_json::Value>, String> {
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

pub async fn create_tree(client: &reqwest::Client, api_base: &str, token: &str, base_tree_sha: &str, tree_entries: Vec<serde_json::Value>) -> Result<String, String> {
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

pub async fn create_commit(client: &reqwest::Client, api_base: &str, token: &str, message: &str, tree_sha: &str, parent_sha: &str) -> Result<String, String> {
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

pub async fn update_branch(client: &reqwest::Client, api_base: &str, token: &str, branch_name: &str, commit_sha: &str) -> Result<(), String> {
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

pub async fn create_pull_request(client: &reqwest::Client, api_base: &str, token: &str, title: &str, head: &str, base: &str, body: &str) -> Result<String, String> {
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
