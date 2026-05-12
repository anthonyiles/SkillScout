use crate::db::AppState;
use crate::models::{Agent, Project};
use rusqlite::params;
use tauri::{command, State};

#[command]
pub fn get_setting(state: State<'_, AppState>, key: String) -> Result<Option<String>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1").map_err(|e| e.to_string())?;
    let mut rows = stmt.query(params![key]).map_err(|e| e.to_string())?;

    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let val: String = row.get(0).map_err(|e| e.to_string())?;
        Ok(Some(val))
    } else {
        Ok(None)
    }
}

#[command]
pub fn set_setting(state: State<'_, AppState>, key: String, value: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = ?2",
        params![key, value],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
pub fn get_agents(state: State<'_, AppState>) -> Result<Vec<Agent>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, name, skills_path, rules_path FROM agents").map_err(|e| e.to_string())?;
    
    let agents_iter = stmt.query_map([], |row| {
        Ok(Agent {
            id: row.get(0)?,
            name: row.get(1)?,
            skills_path: row.get(2)?,
            rules_path: row.get(3)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut agents = Vec::new();
    for agent in agents_iter {
        agents.push(agent.map_err(|e| e.to_string())?);
    }
    Ok(agents)
}

#[command]
pub fn save_agent(state: State<'_, AppState>, agent: Agent) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO agents (id, name, skills_path, rules_path) VALUES (?1, ?2, ?3, ?4) 
         ON CONFLICT(id) DO UPDATE SET name = ?2, skills_path = ?3, rules_path = ?4",
        params![agent.id, agent.name, agent.skills_path, agent.rules_path],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
pub fn delete_agent(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM agents WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
pub fn reset_agents_to_defaults(state: State<'_, AppState>) -> Result<(), String> {
    let mut conn = state.db.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    tx.execute("DELETE FROM agents", []).map_err(|e| e.to_string())?;
    tx.execute_batch("
        INSERT INTO agents (id, name, skills_path, rules_path) VALUES 
            ('cursor', 'Cursor', '.cursor/skills', '.cursor/rules'),
            ('jetbrains', 'JetBrains AI', '.agents/skills', '.agents/rules'),
            ('claude', 'Claude Code', '.claude/skills', '.claude/rules');
    ").map_err(|e| e.to_string())?;
    
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
pub fn get_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, path FROM projects").map_err(|e| e.to_string())?;
    
    let projects_iter = stmt.query_map([], |row| {
        let id: i64 = row.get(0)?;
        let path: String = row.get(1)?;
        Ok((id, path))
    }).map_err(|e| e.to_string())?;

    let mut projects = Vec::new();
    for p in projects_iter {
        let (id, path) = p.map_err(|e| e.to_string())?;
        
        let mut agent_stmt = conn.prepare("SELECT agent_id FROM project_agents WHERE project_id = ?1").map_err(|e| e.to_string())?;
        let agent_ids_iter = agent_stmt.query_map(params![id], |row| row.get::<_, String>(0)).map_err(|e| e.to_string())?;
        
        let mut agent_ids = Vec::new();
        for a_id in agent_ids_iter {
            agent_ids.push(a_id.map_err(|e| e.to_string())?);
        }

        projects.push(Project {
            id: Some(id),
            path,
            agent_ids,
        });
    }
    Ok(projects)
}

#[command]
pub fn save_project(state: State<'_, AppState>, project: Project) -> Result<Project, String> {
    let mut conn = state.db.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    let id = if let Some(pid) = project.id {
        tx.execute("UPDATE projects SET path = ?1 WHERE id = ?2", params![project.path, pid]).map_err(|e| e.to_string())?;
        tx.execute("DELETE FROM project_agents WHERE project_id = ?1", params![pid]).map_err(|e| e.to_string())?;
        pid
    } else {
        tx.execute("INSERT INTO projects (path) VALUES (?1)", params![project.path]).map_err(|e| e.to_string())?;
        tx.last_insert_rowid()
    };

    for agent_id in &project.agent_ids {
        tx.execute("INSERT INTO project_agents (project_id, agent_id) VALUES (?1, ?2)", params![id, agent_id]).map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;
    
    let mut saved_project = project;
    saved_project.id = Some(id);
    Ok(saved_project)
}

#[command]
pub fn delete_project(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM projects WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
    Ok(())
}
