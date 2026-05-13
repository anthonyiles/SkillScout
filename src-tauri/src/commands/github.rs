use std::path::Path;
use crate::utils::auth::load_token;
use crate::github::api::{
    parse_repo_url, get_repo_info, create_branch, create_blobs_for_item,
    create_tree, create_commit, update_branch, create_pull_request, delete_branch
};

#[tauri::command]
pub async fn check_pr_status(app: tauri::AppHandle, pr_url: String) -> Result<serde_json::Value, String> {
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
    let pull_number = path_parts[3]
        .split(&['?', '#'][..])
        .next()
        .unwrap_or(path_parts[3]);
        
    let pull_number: u64 = pull_number
        .parse()
        .map_err(|_| "Invalid PR URL format".to_string())?;

    let api_url = format!("https://api.github.com/repos/{}/{}/pulls/{}", owner, repo, pull_number);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| { eprintln!("HTTP client build error: {}", e); "Failed to initialize network client".to_string() })?;

    let res = client.get(&api_url)
        .header("User-Agent", "SkillScout-App")
        .header("Authorization", format!("Bearer {}", token))
        .send().await.map_err(|e| { eprintln!("GitHub API request error: {}", e); "Failed to connect to GitHub".to_string() })?;

    if !res.status().is_success() {
        return Err(format!("Failed to fetch PR status: {}", res.status()));
    }

    let pr_info: serde_json::Value = res.json().await.map_err(|e| { eprintln!("GitHub API JSON error: {}", e); "Failed to parse GitHub response".to_string() })?;
    let state = pr_info["state"].as_str().unwrap_or("unknown").to_string();
    let merged = pr_info["merged"].as_bool().unwrap_or(false);
    
    Ok(serde_json::json!({
        "state": state,
        "merged": merged
    }))
}

#[tauri::command]
pub async fn promote_item(
    app: tauri::AppHandle,
    repo_url: String,
    item_type: String,
    item_name: String,
    project_path: String,
    sub_folders: Vec<String>,
) -> Result<serde_json::Value, String> {
    let token = load_token(&app)?;
    let (owner, repo) = parse_repo_url(&repo_url).ok_or("Invalid repository URL format.")?;

    let base_path = Path::new(&project_path)
        .canonicalize()
        .map_err(|e| { eprintln!("Path canonicalization error: {}", e); format!("Invalid project path: {}", e) })?;
    let mut item_path = None;
    for folder in sub_folders {
        let candidate = base_path.join(&folder).join(&item_name);
        if candidate.exists() {
            let path = candidate
                .canonicalize()
                .map_err(|e| { eprintln!("Path canonicalization error: {}", e); format!("Failed to resolve item path: {}", e) })?;
            if !path.starts_with(&base_path) {
                return Err("Item path escapes the selected project".into());
            }
            item_path = Some(path);
            break;
        }
    }
    let item_path = item_path.ok_or("Could not locate the item locally.")?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| { eprintln!("HTTP client build error: {}", e); "Failed to initialize network client".to_string() })?;

    let api_base = format!("https://api.github.com/repos/{}/{}", owner, repo);

    let (default_branch, latest_sha) = get_repo_info(&client, &api_base, &token).await?;

    let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
    let safe_name = item_name.replace(|c: char| !c.is_alphanumeric(), "-");
    let new_branch_name = format!("promote-{}-{}", safe_name, timestamp);
    create_branch(&client, &api_base, &token, &new_branch_name, &latest_sha).await?;

    let process = async {
        let tree_entries = create_blobs_for_item(&client, &api_base, &token, &item_path, &item_name, &item_type).await?;
        let new_tree_sha = create_tree(&client, &api_base, &token, &latest_sha, tree_entries).await?;
        let commit_message = format!("Promote {}: {}", item_type, item_name);
        let new_commit_sha = create_commit(&client, &api_base, &token, &commit_message, &new_tree_sha, &latest_sha).await?;
        update_branch(&client, &api_base, &token, &new_branch_name, &new_commit_sha).await?;
        let pr_body = format!("Automated PR to promote the {} `{}` from local environment.", item_type, item_name);
        create_pull_request(&client, &api_base, &token, &commit_message, &new_branch_name, &default_branch, &pr_body).await
    };

    match process.await {
        Ok(html_url) => Ok(serde_json::json!({
            "url": html_url,
            "branch": new_branch_name
        })),
        Err(e) => {
            let _ = delete_branch(&client, &api_base, &token, &new_branch_name).await;
            Err(e)
        }
    }
}
