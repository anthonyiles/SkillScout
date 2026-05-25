use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct FileHash {
    pub name: String,
    pub sha: String,
    pub folder: String,
    pub content: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
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
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
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
    #[serde(rename = "subFolder")]
    pub sub_folder: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn agent_serializes_with_camel_case_field_names() {
        let agent = Agent {
            id: "cursor".to_string(),
            name: "Cursor".to_string(),
            skills_path: ".cursor/skills".to_string(),
            rules_path: ".cursor/rules".to_string(),
        };
        let json = serde_json::to_value(&agent).unwrap();
        assert_eq!(json["id"], "cursor");
        assert_eq!(json["skillsPath"], ".cursor/skills");
        assert_eq!(json["rulesPath"], ".cursor/rules");
        assert!(json.get("skills_path").is_none(), "snake_case key must not appear");
    }

    #[test]
    fn agent_deserializes_from_camel_case() {
        let json = r#"{"id":"cursor","name":"Cursor","skillsPath":".cursor/skills","rulesPath":".cursor/rules"}"#;
        let agent: Agent = serde_json::from_str(json).unwrap();
        assert_eq!(agent.id, "cursor");
        assert_eq!(agent.skills_path, ".cursor/skills");
    }

    #[test]
    fn project_serializes_agent_ids_as_camel_case() {
        let project = Project {
            id: Some(1),
            path: "/home/user/project".to_string(),
            agent_ids: vec!["cursor".to_string()],
        };
        let json = serde_json::to_value(&project).unwrap();
        assert_eq!(json["agentIds"][0], "cursor");
        assert!(json.get("agent_ids").is_none(), "snake_case key must not appear");
    }

    #[test]
    fn sync_task_remove_defaults_to_false() {
        let json = r#"{"source_file":null,"target_dir":"/tmp","file_name":"test.md"}"#;
        let task: SyncTask = serde_json::from_str(json).unwrap();
        assert!(!task.remove);
    }

    #[test]
    fn sync_task_deserializes_all_fields() {
        let json = r#"{"source_file":"/src/test.md","target_dir":"/tmp","file_name":"test.md","remove":true}"#;
        let task: SyncTask = serde_json::from_str(json).unwrap();
        assert_eq!(task.source_file.as_deref(), Some("/src/test.md"));
        assert_eq!(task.target_dir, "/tmp");
        assert_eq!(task.file_name, "test.md");
        assert!(task.remove);
    }

    #[test]
    fn promoted_item_serializes_item_type_and_sub_folder_as_camel_case() {
        let item = PromotedItem {
            id: Some(1),
            name: "my-skill.md".to_string(),
            path: "/path/to/skill".to_string(),
            item_type: "skills".to_string(),
            repository_item_id: None,
            url: None,
            branch: "feat/my-skill".to_string(),
            sub_folder: Some("custom".to_string()),
        };
        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["itemType"], "skills");
        assert_eq!(json["subFolder"], "custom");
        assert!(json.get("item_type").is_none());
        assert!(json.get("sub_folder").is_none());
    }

    #[test]
    fn repository_item_round_trips() {
        let item = RepositoryItem {
            id: "skills-my-skill.md".to_string(),
            name: "my-skill.md".to_string(),
            folder: "skills".to_string(),
            description: Some("A skill".to_string()),
            file_path: "/repo/skills/my-skill.md".to_string(),
            content: "# My Skill".to_string(),
            sha: "abc123".to_string(),
            last_synced: None,
        };
        let json = serde_json::to_string(&item).unwrap();
        let decoded: RepositoryItem = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.id, item.id);
        assert_eq!(decoded.sha, item.sha);
    }
}
