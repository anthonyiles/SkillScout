use crate::db::AppState;
use crate::models::{McpApplyResult, McpApplyTask, McpClash, McpSelection};
use rusqlite::{params, Connection};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};
use tauri::{command, State};

// ── Path resolution ─────────────────────────────────────────────────────────

/// Expand a leading `~` to the user's home directory and validate the result is
/// an absolute path with no parent-dir (`..`) components.
fn resolve_mcp_path(raw: &str) -> Result<PathBuf, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("MCP target path is empty".to_string());
    }

    let expanded: PathBuf = if trimmed == "~" {
        dirs::home_dir().ok_or_else(|| "Could not resolve home directory".to_string())?
    } else if let Some(rest) = trimmed.strip_prefix("~/").or_else(|| trimmed.strip_prefix("~\\")) {
        let home = dirs::home_dir().ok_or_else(|| "Could not resolve home directory".to_string())?;
        home.join(rest)
    } else {
        PathBuf::from(trimmed)
    };

    if !expanded.is_absolute() {
        return Err(format!("MCP target path must be absolute: {}", raw));
    }
    if expanded.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(format!("MCP target path must not contain '..': {}", raw));
    }

    Ok(expanded)
}

// ── mcp.json read/write helpers ──────────────────────────────────────────────

/// Read an mcp.json file and return its top-level object. A missing file yields
/// an empty object. An existing file that is not valid JSON, or whose root is
/// not an object, is an error — we never silently overwrite a malformed file.
fn read_mcp_file(path: &Path) -> Result<Map<String, Value>, String> {
    match std::fs::read_to_string(path) {
        Ok(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return Ok(Map::new());
            }
            let value: Value = serde_json::from_str(trimmed)
                .map_err(|e| format!("{} is not valid JSON: {}", path.display(), e))?;
            match value {
                Value::Object(map) => Ok(map),
                _ => Err(format!("{} root is not a JSON object", path.display())),
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Map::new()),
        Err(e) => Err(format!("Failed to read {}: {}", path.display(), e)),
    }
}

fn write_mcp_file(path: &Path, root: &Map<String, Value>) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory {}: {}", parent.display(), e))?;
    }
    let mut text = serde_json::to_string_pretty(&Value::Object(root.clone()))
        .map_err(|e| format!("Failed to serialise MCP config: {}", e))?;
    text.push('\n');
    std::fs::write(path, text).map_err(|e| format!("Failed to write {}: {}", path.display(), e))
}

/// Borrow the `mcpServers` object from the root, creating it if absent. Errors
/// if it exists but is not an object.
fn mcp_servers_mut<'a>(root: &'a mut Map<String, Value>, path: &Path) -> Result<&'a mut Map<String, Value>, String> {
    let entry = root.entry("mcpServers".to_string()).or_insert_with(|| Value::Object(Map::new()));
    match entry {
        Value::Object(map) => Ok(map),
        _ => Err(format!("{} has a non-object \"mcpServers\" field", path.display())),
    }
}

// ── Apply ────────────────────────────────────────────────────────────────────

/// Pure core of the apply operation, kept separate from the Tauri/spawn_blocking
/// wrapper so it can be unit-tested against real temp files.
pub(crate) fn apply_mcp_tasks(tasks: Vec<McpApplyTask>) -> Result<McpApplyResult, String> {
    let mut result = McpApplyResult::default();

    // Group tasks by resolved target file so each file is read and written once.
    // BTreeMap keeps the processing order deterministic (nice for tests).
    let mut by_file: BTreeMap<PathBuf, Vec<McpApplyTask>> = BTreeMap::new();
    for task in tasks {
        let resolved = resolve_mcp_path(&task.target_path)?;
        by_file.entry(resolved).or_default().push(task);
    }

    for (path, tasks) in by_file {
        let mut root = read_mcp_file(&path)?;
        let mut modified = false;

        // Collect clashes per file without holding the servers borrow across writes.
        let mut file_clashes: Vec<McpClash> = Vec::new();
        {
            let servers = mcp_servers_mut(&mut root, &path)?;
            for task in &tasks {
                if task.remove {
                    if servers.remove(&task.server_key).is_some() {
                        result.removed += 1;
                        modified = true;
                    }
                    continue;
                }

                let incoming: Value = serde_json::from_str(&task.config)
                    .map_err(|e| format!("Invalid MCP config for \"{}\": {}", task.server_key, e))?;

                match servers.get(&task.server_key) {
                    Some(existing) if *existing == incoming => {
                        // Already present and identical — adopt without writing.
                        result.adopted += 1;
                    }
                    Some(existing) if !task.force => {
                        file_clashes.push(McpClash {
                            target_path: task.target_path.clone(),
                            server_key: task.server_key.clone(),
                            existing_config: serde_json::to_string_pretty(existing).unwrap_or_default(),
                            incoming_config: serde_json::to_string_pretty(&incoming).unwrap_or_default(),
                        });
                    }
                    _ => {
                        servers.insert(task.server_key.clone(), incoming);
                        result.applied += 1;
                        modified = true;
                    }
                }
            }
        }

        result.clashes.extend(file_clashes);

        if modified {
            write_mcp_file(&path, &root)?;
        }
    }

    Ok(result)
}

// ── DB helpers for mcp_selections ────────────────────────────────────────────

pub(crate) fn db_get_mcp_selections(conn: &Connection) -> Result<Vec<McpSelection>, String> {
    let mut stmt = conn
        .prepare("SELECT item_id, scope, project_id FROM mcp_selections")
        .map_err(|e| { eprintln!("Failed to prepare MCP selections query: {}", e); "Failed to fetch MCP selections".to_string() })?;

    let iter = stmt
        .query_map([], |row| {
            Ok(McpSelection { item_id: row.get(0)?, scope: row.get(1)?, project_id: row.get(2)? })
        })
        .map_err(|e| { eprintln!("Failed to query MCP selections: {}", e); "Failed to fetch MCP selections".to_string() })?;

    let mut result = Vec::new();
    for sel in iter {
        result.push(sel.map_err(|e| { eprintln!("Corrupt MCP selection row: {}", e); "Corrupt MCP selection data".to_string() })?);
    }
    Ok(result)
}

/// Insert a selection if absent, or remove it if present. Returns the new
/// selected state (`true` = now selected). For global scope, pass `project_id = 0`.
pub(crate) fn db_set_mcp_selection(conn: &Connection, item_id: &str, scope: &str, project_id: i64, selected: bool) -> Result<(), String> {
    if scope != "global" && scope != "project" {
        return Err("Invalid MCP selection scope".to_string());
    }
    if selected {
        conn.execute(
            "INSERT OR IGNORE INTO mcp_selections (item_id, scope, project_id) VALUES (?1, ?2, ?3)",
            params![item_id, scope, project_id],
        ).map_err(|e| { eprintln!("Failed to insert MCP selection: {}", e); "Failed to update MCP selection".to_string() })?;
    } else {
        conn.execute(
            "DELETE FROM mcp_selections WHERE item_id = ?1 AND scope = ?2 AND project_id = ?3",
            params![item_id, scope, project_id],
        ).map_err(|e| { eprintln!("Failed to delete MCP selection: {}", e); "Failed to update MCP selection".to_string() })?;
    }
    Ok(())
}

// ── Tauri commands ───────────────────────────────────────────────────────────

#[command]
pub async fn apply_mcp_servers(tasks: Vec<McpApplyTask>) -> Result<McpApplyResult, String> {
    tauri::async_runtime::spawn_blocking(move || apply_mcp_tasks(tasks))
        .await
        .map_err(|e| { eprintln!("Spawn blocking error: {}", e); "Background task failed".to_string() })?
}

#[command]
pub fn get_mcp_selections(state: State<'_, AppState>) -> Result<Vec<McpSelection>, String> {
    db_get_mcp_selections(&state.lock_conn())
}

#[command]
pub fn set_mcp_selection(state: State<'_, AppState>, item_id: String, scope: String, project_id: i64, selected: bool) -> Result<(), String> {
    db_set_mcp_selection(&state.lock_conn(), &item_id, &scope, project_id, selected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{create_schema, seed_defaults};
    use std::fs;

    fn task(path: &str, key: &str, config: &str) -> McpApplyTask {
        McpApplyTask { target_path: path.to_string(), server_key: key.to_string(), config: config.to_string(), remove: false, force: false }
    }

    // ── apply_mcp_tasks ──────────────────────────────────────────────────────

    #[test]
    fn applies_server_to_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mcp.json");
        let cfg = r#"{"command":"npx","args":["-y","ctx7"]}"#;

        let result = apply_mcp_tasks(vec![task(path.to_str().unwrap(), "context7", cfg)]).unwrap();

        assert_eq!(result.applied, 1);
        assert_eq!(result.adopted, 0);
        assert!(result.clashes.is_empty());

        let written: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(written["mcpServers"]["context7"]["command"], "npx");
    }

    #[test]
    fn preserves_existing_other_servers_and_keys() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mcp.json");
        fs::write(&path, r#"{"otherKey":1,"mcpServers":{"existing":{"command":"foo"}}}"#).unwrap();

        apply_mcp_tasks(vec![task(path.to_str().unwrap(), "context7", r#"{"command":"npx"}"#)]).unwrap();

        let written: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(written["otherKey"], 1);
        assert_eq!(written["mcpServers"]["existing"]["command"], "foo");
        assert_eq!(written["mcpServers"]["context7"]["command"], "npx");
    }

    #[test]
    fn adopts_identical_existing_key_without_clash() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mcp.json");
        fs::write(&path, r#"{"mcpServers":{"context7":{"command":"npx"}}}"#).unwrap();

        let result = apply_mcp_tasks(vec![task(path.to_str().unwrap(), "context7", r#"{"command":"npx"}"#)]).unwrap();

        assert_eq!(result.adopted, 1);
        assert_eq!(result.applied, 0);
        assert!(result.clashes.is_empty());
    }

    #[test]
    fn reports_clash_for_differing_existing_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mcp.json");
        fs::write(&path, r#"{"mcpServers":{"context7":{"command":"old"}}}"#).unwrap();

        let result = apply_mcp_tasks(vec![task(path.to_str().unwrap(), "context7", r#"{"command":"new"}"#)]).unwrap();

        assert_eq!(result.applied, 0);
        assert_eq!(result.clashes.len(), 1);
        assert_eq!(result.clashes[0].server_key, "context7");

        // File must be untouched when a clash is reported.
        let written: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(written["mcpServers"]["context7"]["command"], "old");
    }

    #[test]
    fn force_overwrites_differing_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mcp.json");
        fs::write(&path, r#"{"mcpServers":{"context7":{"command":"old"}}}"#).unwrap();

        let mut t = task(path.to_str().unwrap(), "context7", r#"{"command":"new"}"#);
        t.force = true;
        let result = apply_mcp_tasks(vec![t]).unwrap();

        assert_eq!(result.applied, 1);
        assert!(result.clashes.is_empty());

        let written: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(written["mcpServers"]["context7"]["command"], "new");
    }

    #[test]
    fn removes_server_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mcp.json");
        fs::write(&path, r#"{"mcpServers":{"context7":{"command":"npx"},"keep":{"command":"x"}}}"#).unwrap();

        let mut t = task(path.to_str().unwrap(), "context7", "");
        t.remove = true;
        let result = apply_mcp_tasks(vec![t]).unwrap();

        assert_eq!(result.removed, 1);

        let written: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert!(written["mcpServers"].get("context7").is_none());
        assert_eq!(written["mcpServers"]["keep"]["command"], "x");
    }

    #[test]
    fn removing_absent_key_is_noop() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mcp.json");
        fs::write(&path, r#"{"mcpServers":{}}"#).unwrap();

        let mut t = task(path.to_str().unwrap(), "ghost", "");
        t.remove = true;
        let result = apply_mcp_tasks(vec![t]).unwrap();

        assert_eq!(result.removed, 0);
    }

    #[test]
    fn errors_on_malformed_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mcp.json");
        fs::write(&path, "{ not json").unwrap();

        let err = apply_mcp_tasks(vec![task(path.to_str().unwrap(), "context7", r#"{"command":"npx"}"#)]).unwrap_err();
        assert!(err.contains("not valid JSON"));
    }

    #[test]
    fn rejects_path_traversal() {
        let err = apply_mcp_tasks(vec![task("/tmp/../etc/mcp.json", "x", "{}")]).unwrap_err();
        assert!(err.contains(".."));
    }

    // ── mcp_selections DB ────────────────────────────────────────────────────

    fn open_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        seed_defaults(&conn).unwrap();
        conn
    }

    fn insert_mcp_item(conn: &Connection, id: &str) {
        conn.execute(
            "INSERT INTO repository_items (id, name, folder, file_path, content, sha) VALUES (?1, ?2, 'mcp-servers', ?3, ?4, 'sha')",
            params![id, format!("{}.json", id), format!("/mcp-servers/{}.json", id), "{}"],
        ).unwrap();
    }

    #[test]
    fn mcp_selection_toggle_round_trip() {
        let conn = open_test_db();
        insert_mcp_item(&conn, "mcp-servers-context7.json");

        db_set_mcp_selection(&conn, "mcp-servers-context7.json", "global", 0, true).unwrap();
        let sels = db_get_mcp_selections(&conn).unwrap();
        assert_eq!(sels.len(), 1);
        assert_eq!(sels[0].scope, "global");

        db_set_mcp_selection(&conn, "mcp-servers-context7.json", "global", 0, false).unwrap();
        assert!(db_get_mcp_selections(&conn).unwrap().is_empty());
    }

    #[test]
    fn mcp_selection_global_and_project_coexist() {
        let conn = open_test_db();
        insert_mcp_item(&conn, "mcp-servers-context7.json");
        conn.execute("INSERT INTO projects (path) VALUES ('/home/user/proj')", []).unwrap();
        let pid = conn.last_insert_rowid();

        db_set_mcp_selection(&conn, "mcp-servers-context7.json", "global", 0, true).unwrap();
        db_set_mcp_selection(&conn, "mcp-servers-context7.json", "project", pid, true).unwrap();

        assert_eq!(db_get_mcp_selections(&conn).unwrap().len(), 2);
    }

    #[test]
    fn mcp_selection_rejects_invalid_scope() {
        let conn = open_test_db();
        insert_mcp_item(&conn, "mcp-servers-context7.json");
        assert!(db_set_mcp_selection(&conn, "mcp-servers-context7.json", "bogus", 0, true).is_err());
    }
}
