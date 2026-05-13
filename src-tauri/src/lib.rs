mod app_state;
mod models;
mod utils;
mod github;
mod commands;

use app_state::AppState;
use commands::auth::{check_github_auth, logout_github, poll_github_token, start_github_device_flow};
use commands::github::{check_pr_status, promote_item};
use commands::sync::{apply_skills, check_existing, get_project_files, sync_repo};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenvy::dotenv().ok();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            sync_repo, check_existing, apply_skills, get_project_files,
            start_github_device_flow, poll_github_token, check_github_auth, logout_github,
            promote_item, check_pr_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
