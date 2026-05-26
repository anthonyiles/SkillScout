use crate::db::AppState;
use crate::models::{Agent, Project};
use rusqlite::{params, Connection};
use tauri::{command, State};

fn validate_project_path(path: &str) -> Result<(), String> {
    if path.trim().is_empty() {
        return Err("Project path cannot be empty".to_string());
    }

    let path_ref = std::path::Path::new(path);

    if !path_ref.is_absolute() {
        return Err("Project path must be an absolute path".to_string());
    }

    if path_ref.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Err("Project path must not contain '..' components".to_string());
    }

    Ok(())
}

// ── Inner DB helpers (pub(crate) so tests can call them directly) ───────────

pub(crate) fn db_get_setting(conn: &Connection, key: &str) -> Result<Option<String>, String> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1").map_err(|e| e.to_string())?;
    let mut rows = stmt.query(params![key]).map_err(|e| e.to_string())?;
    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        Ok(Some(row.get(0).map_err(|e| e.to_string())?))
    } else {
        Ok(None)
    }
}

pub(crate) fn db_set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = ?2",
        params![key, value],
    ).map_err(|e| { eprintln!("Failed to save setting: {}", e); "Failed to save setting".to_string() })?;
    Ok(())
}

pub(crate) fn db_get_agents(conn: &Connection) -> Result<Vec<Agent>, String> {
    let mut stmt = conn.prepare("SELECT id, name, skills_path, rules_path FROM agents")
        .map_err(|e| { eprintln!("Failed to prepare agents query: {}", e); "Failed to load agents".to_string() })?;

    let agents_iter = stmt.query_map([], |row| {
        Ok(Agent {
            id: row.get(0)?,
            name: row.get(1)?,
            skills_path: row.get(2)?,
            rules_path: row.get(3)?,
        })
    }).map_err(|e| { eprintln!("Failed to query agents: {}", e); "Failed to load agents".to_string() })?;

    let mut agents = Vec::new();
    for agent in agents_iter {
        agents.push(agent.map_err(|e| { eprintln!("Corrupt agent row: {}", e); "Corrupt agent data".to_string() })?);
    }
    Ok(agents)
}

pub(crate) fn db_save_agent(conn: &Connection, agent: &Agent) -> Result<(), String> {
    conn.execute(
        "INSERT INTO agents (id, name, skills_path, rules_path) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(id) DO UPDATE SET name = ?2, skills_path = ?3, rules_path = ?4",
        params![agent.id, agent.name, agent.skills_path, agent.rules_path],
    ).map_err(|e| { eprintln!("Failed to save agent: {}", e); "Failed to save agent".to_string() })?;
    Ok(())
}

pub(crate) fn db_delete_agent(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM agents WHERE id = ?1", params![id])
        .map_err(|e| { eprintln!("Failed to delete agent: {}", e); "Failed to delete agent".to_string() })?;
    Ok(())
}

pub(crate) fn db_reset_agents(conn: &mut Connection) -> Result<(), String> {
    let tx = conn.transaction()
        .map_err(|e| { eprintln!("Failed to begin transaction: {}", e); "Database busy".to_string() })?;

    tx.execute("DELETE FROM agents", [])
        .map_err(|e| { eprintln!("Failed to clear agents: {}", e); "Failed to reset agents".to_string() })?;
    tx.execute_batch("
        INSERT INTO agents (id, name, skills_path, rules_path) VALUES
            ('cursor', 'Cursor', '.cursor/skills', '.cursor/rules'),
            ('jetbrains', 'JetBrains AI', '.agents/skills', '.agents/rules'),
            ('claude', 'Claude Code', '.claude/skills', '.claude/rules');
    ").map_err(|e| { eprintln!("Failed to seed agents: {}", e); "Failed to reset agents".to_string() })?;

    tx.commit().map_err(|e| { eprintln!("Failed to commit reset: {}", e); "Failed to save changes".to_string() })?;
    Ok(())
}

pub(crate) fn db_get_projects(conn: &Connection) -> Result<Vec<Project>, String> {
    let mut stmt = conn.prepare(
        "SELECT p.id, p.path, pa.agent_id \
         FROM projects p \
         LEFT JOIN project_agents pa ON p.id = pa.project_id"
    ).map_err(|e| { eprintln!("Failed to prepare projects query: {}", e); "Failed to fetch projects".to_string() })?;

    let rows = stmt.query_map([], |row| {
        let id: i64 = row.get(0)?;
        let path: String = row.get(1)?;
        let agent_id: Option<String> = row.get(2)?;
        Ok((id, path, agent_id))
    }).map_err(|e| { eprintln!("Failed to query projects: {}", e); "Failed to fetch projects".to_string() })?;

    let mut map: std::collections::HashMap<i64, Project> = std::collections::HashMap::new();
    let mut order: Vec<i64> = Vec::new();

    for row in rows {
        let (id, path, agent_id) = row.map_err(|e| { eprintln!("Corrupt project row: {}", e); "Corrupt project data".to_string() })?;
        let project = map.entry(id).or_insert_with(|| {
            order.push(id);
            Project { id: Some(id), path, agent_ids: Vec::new() }
        });
        if let Some(agent_id_value) = agent_id {
            project.agent_ids.push(agent_id_value);
        }
    }

    Ok(order.into_iter().filter_map(|id| map.remove(&id)).collect())
}

pub(crate) fn db_save_project(conn: &mut Connection, project: &Project) -> Result<Project, String> {
    validate_project_path(&project.path)?;

    let tx = conn.transaction()
        .map_err(|e| { eprintln!("Failed to begin transaction: {}", e); "Database busy".to_string() })?;

    let id = if let Some(pid) = project.id {
        tx.execute("UPDATE projects SET path = ?1 WHERE id = ?2", params![project.path, pid])
            .map_err(|e| { eprintln!("Failed to update project: {}", e); "Failed to update project".to_string() })?;
        tx.execute("DELETE FROM project_agents WHERE project_id = ?1", params![pid])
            .map_err(|e| { eprintln!("Failed to clear project agents: {}", e); "Failed to update project agents".to_string() })?;
        pid
    } else {
        tx.execute("INSERT INTO projects (path) VALUES (?1)", params![project.path])
            .map_err(|e| { eprintln!("Failed to insert project: {}", e); "Failed to create project".to_string() })?;
        tx.last_insert_rowid()
    };

    for agent_id in &project.agent_ids {
        tx.execute("INSERT INTO project_agents (project_id, agent_id) VALUES (?1, ?2)", params![id, agent_id])
            .map_err(|e| { eprintln!("Failed to assign agent: {}", e); "Failed to assign agent to project".to_string() })?;
    }

    tx.commit().map_err(|e| { eprintln!("Failed to commit project save: {}", e); "Failed to save changes".to_string() })?;

    let mut saved = project.clone();
    saved.id = Some(id);
    Ok(saved)
}

pub(crate) fn db_delete_project(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM projects WHERE id = ?1", params![id])
        .map_err(|e| { eprintln!("Failed to delete project: {}", e); "Failed to delete project".to_string() })?;
    Ok(())
}

// ── Tauri commands (thin wrappers over DB helpers) ───────────────────────────

#[command]
pub fn get_setting(state: State<'_, AppState>, key: String) -> Result<Option<String>, String> {
    db_get_setting(&state.lock_conn(), &key)
}

#[command]
pub fn set_setting(state: State<'_, AppState>, key: String, value: String) -> Result<(), String> {
    db_set_setting(&state.lock_conn(), &key, &value)
}

#[command]
pub fn get_agents(state: State<'_, AppState>) -> Result<Vec<Agent>, String> {
    db_get_agents(&state.lock_conn())
}

#[command]
pub fn save_agent(state: State<'_, AppState>, agent: Agent) -> Result<(), String> {
    db_save_agent(&state.lock_conn(), &agent)
}

#[command]
pub fn delete_agent(state: State<'_, AppState>, id: String) -> Result<(), String> {
    db_delete_agent(&state.lock_conn(), &id)
}

#[command]
pub fn reset_agents_to_defaults(state: State<'_, AppState>) -> Result<(), String> {
    db_reset_agents(&mut state.lock_conn())
}

#[command]
pub fn get_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    db_get_projects(&state.lock_conn())
}

#[command]
pub fn save_project(state: State<'_, AppState>, project: Project) -> Result<Project, String> {
    db_save_project(&mut state.lock_conn(), &project)
}

#[command]
pub fn delete_project(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    db_delete_project(&state.lock_conn(), id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{create_schema, seed_defaults};

    fn open_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        seed_defaults(&conn).unwrap();
        conn
    }

    // Returns a platform-appropriate absolute path for use in tests.
    // Windows does not recognise Unix-rooted paths as absolute.
    fn abs_test_path(name: &str) -> String {
        #[cfg(windows)]
        { format!("C:\\Users\\{}", name) }
        #[cfg(not(windows))]
        { format!("/home/user/{}", name) }
    }

    // ── Settings ──────────────────────────────────────────────────────────────

    #[test]
    fn set_and_get_setting_round_trip() {
        let conn = open_test_db();
        db_set_setting(&conn, "repoUrl", "https://github.com/org/repo").unwrap();
        let val = db_get_setting(&conn, "repoUrl").unwrap();
        assert_eq!(val, Some("https://github.com/org/repo".to_string()));
    }

    #[test]
    fn get_setting_returns_none_for_missing_key() {
        let conn = open_test_db();
        let val = db_get_setting(&conn, "nonexistent").unwrap();
        assert_eq!(val, None);
    }

    #[test]
    fn set_setting_overwrites_existing_value() {
        let conn = open_test_db();
        db_set_setting(&conn, "key", "first").unwrap();
        db_set_setting(&conn, "key", "second").unwrap();
        let val = db_get_setting(&conn, "key").unwrap();
        assert_eq!(val, Some("second".to_string()));
    }

    // ── Agents ────────────────────────────────────────────────────────────────

    #[test]
    fn get_agents_returns_seeded_defaults() {
        let conn = open_test_db();
        let agents = db_get_agents(&conn).unwrap();
        assert_eq!(agents.len(), 3);
        assert!(agents.iter().any(|a| a.id == "cursor"));
        assert!(agents.iter().any(|a| a.id == "jetbrains"));
        assert!(agents.iter().any(|a| a.id == "claude"));
    }

    #[test]
    fn save_agent_inserts_new_agent() {
        let conn = open_test_db();
        let agent = Agent {
            id: "custom".to_string(),
            name: "Custom Agent".to_string(),
            skills_path: ".custom/skills".to_string(),
            rules_path: ".custom/rules".to_string(),
        };
        db_save_agent(&conn, &agent).unwrap();
        let agents = db_get_agents(&conn).unwrap();
        assert!(agents.iter().any(|a| a.id == "custom"));
    }

    #[test]
    fn save_agent_updates_existing_agent() {
        let conn = open_test_db();
        let mut agent = Agent {
            id: "cursor".to_string(),
            name: "Cursor Updated".to_string(),
            skills_path: ".cursor/skills".to_string(),
            rules_path: ".cursor/rules".to_string(),
        };
        db_save_agent(&conn, &agent).unwrap();
        agent.name = "Cursor Updated Again".to_string();
        db_save_agent(&conn, &agent).unwrap();

        let agents = db_get_agents(&conn).unwrap();
        let cursor = agents.iter().find(|a| a.id == "cursor").unwrap();
        assert_eq!(cursor.name, "Cursor Updated Again");
    }

    #[test]
    fn delete_agent_removes_it() {
        let conn = open_test_db();
        db_delete_agent(&conn, "cursor").unwrap();
        let agents = db_get_agents(&conn).unwrap();
        assert!(!agents.iter().any(|a| a.id == "cursor"));
    }

    #[test]
    fn reset_agents_restores_three_defaults() {
        let mut conn = open_test_db();
        db_delete_agent(&conn, "cursor").unwrap();
        db_delete_agent(&conn, "jetbrains").unwrap();
        db_reset_agents(&mut conn).unwrap();
        let agents = db_get_agents(&conn).unwrap();
        assert_eq!(agents.len(), 3);
    }

    // ── Projects ──────────────────────────────────────────────────────────────

    #[test]
    fn save_project_inserts_and_assigns_id() {
        let mut conn = open_test_db();
        let path = abs_test_path("project");
        let project = Project { id: None, path: path.clone(), agent_ids: vec!["cursor".to_string()] };
        let saved = db_save_project(&mut conn, &project).unwrap();
        assert!(saved.id.is_some());
        assert_eq!(saved.path, path);
        assert_eq!(saved.agent_ids, vec!["cursor"]);
    }

    #[test]
    fn save_project_rejects_relative_path() {
        let mut conn = open_test_db();
        let project = Project { id: None, path: "relative/path".to_string(), agent_ids: vec![] };
        assert!(db_save_project(&mut conn, &project).is_err());
    }

    #[test]
    fn get_projects_returns_saved_project() {
        let mut conn = open_test_db();
        let path = abs_test_path("alpha");
        let project = Project { id: None, path: path.clone(), agent_ids: vec![] };
        db_save_project(&mut conn, &project).unwrap();
        let projects = db_get_projects(&conn).unwrap();
        assert!(projects.iter().any(|p| p.path == path));
    }

    #[test]
    fn delete_project_removes_it() {
        let mut conn = open_test_db();
        let path = abs_test_path("beta");
        let project = Project { id: None, path: path.clone(), agent_ids: vec![] };
        let saved = db_save_project(&mut conn, &project).unwrap();
        db_delete_project(&conn, saved.id.unwrap()).unwrap();
        let projects = db_get_projects(&conn).unwrap();
        assert!(!projects.iter().any(|p| p.path == path));
    }

    // ── Path validation ───────────────────────────────────────────────────────

    #[test]
    fn validate_project_path_accepts_absolute_path() {
        assert!(validate_project_path(&abs_test_path("project")).is_ok());
    }

    #[test]
    fn validate_project_path_rejects_empty() {
        assert!(validate_project_path("").is_err());
    }

    #[test]
    fn validate_project_path_rejects_relative_path() {
        assert!(validate_project_path("relative/path").is_err());
    }

    #[test]
    fn validate_project_path_rejects_traversal() {
        #[cfg(windows)]
        let path = "C:\\Users\\user\\..\\..\\Windows";
        #[cfg(not(windows))]
        let path = "/home/user/../../../etc";
        assert!(validate_project_path(path).is_err());
    }
}
