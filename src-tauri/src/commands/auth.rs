use tauri::State;
use crate::app_state::AppState;
use crate::models::{DeviceAuthResponse, TokenResponse};
use crate::utils::auth::{load_token, save_token};

const GITHUB_DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
const GITHUB_OAUTH_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const GITHUB_USER_URL: &str = "https://api.github.com/user";

#[tauri::command]
pub async fn start_github_device_flow(state: State<'_, AppState>) -> Result<DeviceAuthResponse, String> {
    let client_id = std::env::var("GITHUB_CLIENT_ID").unwrap_or_default();
    if client_id.is_empty() || client_id == "your_sandbox_client_id_here" {
        return Err("GitHub Client ID is missing or invalid in configuration.".to_string());
    }

    let res = state.http
        .post(GITHUB_DEVICE_CODE_URL)
        .header("Accept", "application/json")
        .query(&[("client_id", &client_id), ("scope", &"repo".to_string())])
        .send().await
        .map_err(|_| "Network error while starting authentication flow.".to_string())?;

    let status = res.status();
    let body_text = res.text().await
        .map_err(|_| "Failed to read response body.".to_string())?;

    if !status.is_success() {
        if let Ok(err_json) = serde_json::from_str::<serde_json::Value>(&body_text) {
            if let Some(desc) = err_json.get("error_description").and_then(|v| v.as_str()) {
                return Err(desc.to_string());
            } else if let Some(err) = err_json.get("error").and_then(|v| v.as_str()) {
                return Err(err.to_string());
            }
        }
        return Err(format!("GitHub API error: {}", status));
    }

    let auth_res = serde_json::from_str::<DeviceAuthResponse>(&body_text)
        .map_err(|_| "Failed to process GitHub's response.".to_string())?;

    if auth_res.error.is_some() {
        return Err(auth_res.error_description.clone().unwrap_or_else(|| auth_res.error.clone().unwrap()));
    }

    Ok(auth_res)
}

#[tauri::command]
pub async fn poll_github_token(state: State<'_, AppState>, device_code: String) -> Result<TokenResponse, String> {
    let client_id = std::env::var("GITHUB_CLIENT_ID").unwrap_or_default();
    if client_id.is_empty() || client_id == "your_sandbox_client_id_here" {
        return Err("GitHub Client ID is missing or invalid in configuration.".to_string());
    }

    let res = state.http
        .post(GITHUB_OAUTH_TOKEN_URL)
        .header("Accept", "application/json")
        .query(&[
            ("client_id", &client_id),
            ("device_code", &device_code),
            ("grant_type", &"urn:ietf:params:oauth:grant-type:device_code".to_string()),
        ])
        .send().await
        .map_err(|_| "Network error while polling for access token.".to_string())?;

    let token_res = res.json::<TokenResponse>().await
        .map_err(|_| "Failed to process GitHub's response.".to_string())?;

    if let Some(token) = &token_res.access_token {
        save_token(token)?;
    }

    Ok(token_res)
}

#[tauri::command]
pub async fn check_github_auth(state: State<'_, AppState>) -> Result<bool, String> {
    let token = match load_token() {
        Ok(t) => t,
        Err(_) => return Ok(false),
    };

    let res = state.http
        .get(GITHUB_USER_URL)
        .header("User-Agent", "SkillScout-App")
        .header("Authorization", format!("Bearer {}", token))
        .send().await
        .map_err(|_| "Network error while verifying GitHub connection.".to_string())?;

    Ok(res.status().is_success())
}

#[tauri::command]
pub async fn logout_github() -> Result<(), String> {
    keyring::Entry::new("skillscout", "github_token")
        .map_err(|e| format!("Failed to access keyring: {}", e))?
        .delete_credential()
        .map_err(|e| format!("Failed to remove credential: {}", e))
}
