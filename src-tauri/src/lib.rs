mod db;
mod error;
mod models;
mod utils;
mod github;
mod commands;

use commands::auth::{check_github_auth, logout_github, poll_github_token, start_github_device_flow};
use commands::github::{check_pr_status, promote_item};
use commands::sync::{apply_skills, check_existing, get_project_file_hashes, get_project_files, sync_repo};
use commands::state::{get_setting, set_setting, get_agents, save_agent, delete_agent, reset_agents_to_defaults, get_projects, save_project, delete_project};
use commands::items::{get_repository_items, get_item_selections, toggle_item_selection, update_applied_sha, get_promoted_items, add_promoted_item, remove_promoted_item};
use tauri::Manager;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenvy::dotenv().ok(); // Load environment variables from .env

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let conn = db::initialize_database(app.handle())?;
            app.manage(db::AppState {
                db: Mutex::new(conn),
            });

            // Start background sync loop
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(30 * 60)).await; // Every 30 minutes
                    
                    // Check if a repo is configured
                    let repo_url = {
                        let state: tauri::State<'_, db::AppState> = handle.state();
                        let conn = state.lock_conn();
                        let mut stmt = match conn.prepare("SELECT value FROM settings WHERE key = 'repoUrl'") {
                            Ok(s) => s,
                            Err(_) => continue,
                        };
                        
                        let url: Result<String, _> = stmt.query_row([], |row| row.get(0));
                        url.unwrap_or_default()
                    };

                    if !repo_url.is_empty() {
                        let state = handle.state::<db::AppState>();
                        if let Err(e) = commands::sync::sync_repo(handle.clone(), state, repo_url.clone()).await {
                            eprintln!("[background sync] failed to sync repo '{}': {}", repo_url, e);
                        }
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            sync_repo, check_existing, apply_skills, get_project_files, get_project_file_hashes,
            start_github_device_flow, poll_github_token, check_github_auth, logout_github, promote_item, check_pr_status,
            get_setting, set_setting, get_agents, save_agent, delete_agent, reset_agents_to_defaults, get_projects, save_project, delete_project,
            get_repository_items, get_item_selections, toggle_item_selection, update_applied_sha, get_promoted_items, add_promoted_item, remove_promoted_item
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
