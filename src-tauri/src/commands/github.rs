use std::path::Path;
use tauri::State;
use crate::app_state::AppState;
use crate::models::PrStatus;
use crate::utils::auth::load_token;

use crate::github::api::{
    parse_repo_url, get_repo_info, create_branch, create_blobs_for_item,
    create_tree, create_commit, update_branch, create_pull_request, delete_branch
};

#[tauri::command]
pub async fn check_pr_status(state: State<'_, AppState>, pr_url: String) -> Result<PrStatus, String> {
    let token = load_token()?;

    let parts: Vec<&str> = pr_url.split("github.com/").collect();
    if parts.len() < 2 {
        return Err("Invalid PR URL.".into());
    }
    let path_parts: Vec<&str> = parts[1].split('/').collect();
    if path_parts.len() < 4 || path_parts[2] != "pull" {
        return Err("Invalid PR URL format.".into());
    }

    let owner = path_parts[0];
    let repo = path_parts[1];
    let pull_number = path_parts[3].split(&['?', '#'][..]).next().unwrap_or(path_parts[3]);
    let pull_number: u64 = pull_number.parse().map_err(|_| "Invalid PR number in URL.".to_string())?;

    let api_url = format!("https://api.github.com/repos/{}/{}/pulls/{}", owner, repo, pull_number);

    let res = state.http
        .get(&api_url)
        .header("User-Agent", "SkillScout-App")
        .header("Authorization", format!("Bearer {}", token))
        .send().await
        .map_err(|_| "Failed to connect to GitHub.".to_string())?;

    if !res.status().is_success() {
        return Err(format!("Failed to fetch PR status: {}", res.status()));
    }

    let pr_info: serde_json::Value = res.json().await
        .map_err(|_| "Failed to parse GitHub response.".to_string())?;

    Ok(PrStatus {
        state: pr_info["state"].as_str().unwrap_or("unknown").to_string(),
        merged: pr_info["merged"].as_bool().unwrap_or(false),
    })
}

#[tauri::command]
pub async fn promote_item(
    state: State<'_, AppState>,
    repo_url: String,
    item_type: String,
    item_name: String,
    project_path: String,
    sub_folders: Vec<String>,
) -> Result<String, String> {
    let token = load_token()?;
    let (owner, repo) = parse_repo_url(&repo_url).ok_or("Invalid repository URL format.")?;

    let base_path = Path::new(&project_path)
        .canonicalize()
        .map_err(|e| format!("Invalid project path: {}", e))?;
    let mut item_path = None;
    for folder in sub_folders {
        let candidate = base_path.join(&folder).join(&item_name);
        if candidate.exists() {
            let path = candidate.canonicalize()
                .map_err(|e| format!("Failed to resolve item path: {}", e))?;
            if !path.starts_with(&base_path) {
                return Err("Item path escapes the selected project.".into());
            }
            item_path = Some(path);
            break;
        }
    }
    let item_path = item_path.ok_or("Could not locate the item locally.")?;

    let api_base = format!("https://api.github.com/repos/{}/{}", owner, repo);
    let (default_branch, latest_sha) = get_repo_info(&state.http, &api_base, &token).await?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let safe_name = item_name.replace(|c: char| !c.is_alphanumeric(), "-");
    let new_branch_name = format!("promote-{}-{}", safe_name, timestamp);
    create_branch(&state.http, &api_base, &token, &new_branch_name, &latest_sha).await?;

    let process = async {
        let tree_entries = create_blobs_for_item(&state.http, &api_base, &token, &item_path, &item_name, &item_type).await?;
        let new_tree_sha = create_tree(&state.http, &api_base, &token, &latest_sha, tree_entries).await?;
        let commit_message = format!("Promote {}: {}", item_type, item_name);
        let new_commit_sha = create_commit(&state.http, &api_base, &token, &commit_message, &new_tree_sha, &latest_sha).await?;
        update_branch(&state.http, &api_base, &token, &new_branch_name, &new_commit_sha).await?;
        let pr_body = format!("Automated PR to promote the {} `{}` from local environment.", item_type, item_name);
        create_pull_request(&state.http, &api_base, &token, &commit_message, &new_branch_name, &default_branch, &pr_body).await
    };

    match process.await {
        Ok(html_url) => Ok(html_url),
        Err(e) => {
            let _ = delete_branch(&state.http, &api_base, &token, &new_branch_name).await;
            Err(e)
        }
    }
}
