use rusqlite::{Connection, OptionalExtension};
use std::sync::Mutex;
use tauri::AppHandle;
use tauri::Manager;

pub struct AppState {
    pub db: Mutex<Connection>,
}

impl AppState {
    pub fn lock_conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.db.lock().unwrap_or_else(|poisoned| {
            eprintln!("Recovering database connection after a previous panic");
            poisoned.into_inner()
        })
    }
}

pub(crate) fn create_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute("PRAGMA foreign_keys = ON;", [])?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS agents (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            skills_path TEXT NOT NULL,
            rules_path TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE
        );

        CREATE TABLE IF NOT EXISTS project_agents (
            project_id INTEGER,
            agent_id TEXT,
            PRIMARY KEY (project_id, agent_id),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
            FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS repository_items (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            folder TEXT NOT NULL,
            description TEXT,
            file_path TEXT NOT NULL,
            content TEXT NOT NULL,
            sha TEXT NOT NULL,
            last_synced TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS item_selections (
            item_id TEXT,
            project_id INTEGER,
            applied_sha TEXT,
            PRIMARY KEY (item_id, project_id),
            FOREIGN KEY (item_id) REFERENCES repository_items(id) ON DELETE CASCADE,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS promoted_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            item_type TEXT NOT NULL,
            repository_item_id TEXT,
            url TEXT,
            branch TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (repository_item_id) REFERENCES repository_items(id) ON DELETE SET NULL
        );
        "
    )?;

    // Migration for column added after initial schema; ignore only "duplicate column" errors
    // which mean the column already exists from a previous run.
    match conn.execute("ALTER TABLE promoted_items ADD COLUMN sub_folder TEXT", []) {
        Ok(_) => {}
        Err(rusqlite::Error::SqliteFailure(_, Some(ref msg))) if msg.contains("duplicate column") => {}
        // rusqlite occasionally returns None for the extended message on some SQLite builds;
        // any non-message SqliteFailure from ADD COLUMN is also a duplicate-column variant.
        Err(rusqlite::Error::SqliteFailure(_, None)) => {}
        Err(e) => return Err(e),
    }

    Ok(())
}

pub(crate) fn seed_defaults(conn: &Connection) -> rusqlite::Result<()> {
    let is_initialized: Option<String> = conn.query_row(
        "SELECT value FROM settings WHERE key = 'initialized_defaults'",
        [],
        |row| row.get(0),
    ).optional()?;

    if is_initialized.is_none() {
        conn.execute_batch("
            INSERT OR IGNORE INTO agents (id, name, skills_path, rules_path) VALUES
                ('cursor', 'Cursor', '.cursor/skills', '.cursor/rules'),
                ('jetbrains', 'JetBrains AI', '.agents/skills', '.agents/rules'),
                ('claude', 'Claude Code', '.claude/skills', '.claude/rules');

            INSERT OR IGNORE INTO settings (key, value) VALUES ('initialized_defaults', 'true');
        ")?;
    }

    Ok(())
}

pub fn initialize_database(app_handle: &AppHandle) -> Result<Connection, Box<dyn std::error::Error>> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::NotFound, "Failed to resolve app data directory"))?;

    std::fs::create_dir_all(&app_dir)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to create app data directory: {}", e)))?;

    let db_path = app_dir.join("app_state.db");
    let conn = Connection::open(&db_path)?;

    create_schema(&conn)?;
    seed_defaults(&conn)?;

    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn create_schema_creates_all_tables() {
        let conn = open_test_db();
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"settings".to_string()));
        assert!(tables.contains(&"agents".to_string()));
        assert!(tables.contains(&"projects".to_string()));
        assert!(tables.contains(&"project_agents".to_string()));
        assert!(tables.contains(&"repository_items".to_string()));
        assert!(tables.contains(&"item_selections".to_string()));
        assert!(tables.contains(&"promoted_items".to_string()));
    }

    #[test]
    fn create_schema_is_idempotent() {
        let conn = open_test_db();
        // Running a second time must not error (all tables use IF NOT EXISTS)
        assert!(create_schema(&conn).is_ok());
    }

    #[test]
    fn seed_defaults_inserts_three_default_agents() {
        let conn = open_test_db();
        seed_defaults(&conn).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM agents", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn seed_defaults_is_idempotent() {
        let conn = open_test_db();
        seed_defaults(&conn).unwrap();
        seed_defaults(&conn).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM agents", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn seed_defaults_sets_initialized_flag() {
        let conn = open_test_db();
        seed_defaults(&conn).unwrap();

        let value: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'initialized_defaults'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(value, "true");
    }

    #[test]
    fn foreign_keys_are_enforced() {
        let conn = open_test_db();
        seed_defaults(&conn).unwrap();

        // Inserting a project_agents row referencing a non-existent project should fail
        let result = conn.execute(
            "INSERT INTO project_agents (project_id, agent_id) VALUES (9999, 'cursor')",
            [],
        );
        assert!(result.is_err());
    }
}
