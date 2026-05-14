use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct FileHash {
    pub name: String,
    pub sha: String,
}

#[derive(Serialize, Deserialize)]
pub struct DeviceAuthResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
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

#[derive(Serialize, Deserialize, Clone)]
pub struct Agent {
    pub id: String,
    pub name: String,
    #[serde(rename = "skillsPath")]
    pub skills_path: String,
    #[serde(rename = "rulesPath")]
    pub rules_path: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: Option<i64>,
    pub path: String,
    #[serde(rename = "agentIds")]
    pub agent_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RepositoryItem {
    pub id: String,
    pub name: String,
    pub folder: String,
    pub description: Option<String>,
    pub file_path: String,
    pub content: String,
    pub sha: String,
    pub last_synced: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ItemSelection {
    pub item_id: String,
    pub project_id: i64,
    pub applied_sha: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PromotedItem {
    pub id: Option<i64>,
    pub name: String,
    pub path: String,
    #[serde(rename = "itemType")]
    pub item_type: String,
    pub repository_item_id: Option<String>,
    pub url: Option<String>,
    pub branch: String,
}
