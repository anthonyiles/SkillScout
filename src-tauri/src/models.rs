use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub folder: String,
    pub description: String,
    pub file_path: String,
    pub content: String,
}

#[derive(Serialize, Deserialize)]
pub struct DeviceAuthResponse {
    #[serde(default)]
    pub device_code: String,
    #[serde(default)]
    pub user_code: String,
    #[serde(default)]
    pub verification_uri: String,
    #[serde(default)]
    pub expires_in: u64,
    #[serde(default)]
    pub interval: u64,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: Option<String>,
    pub token_type: Option<String>,
    pub scope: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

#[derive(Deserialize)]
pub struct SyncTask {
    pub source_file: Option<String>,
    pub target_dir: String,
    pub file_name: String,
    #[serde(default)]
    pub remove: bool,
}
