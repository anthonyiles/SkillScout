use rusqlite::Connection;
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

pub fn initialize_database(app_handle: &AppHandle) -> Result<Connection, Box<dyn std::error::Error>> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::NotFound, "Failed to resolve app data directory"))?;

    std::fs::create_dir_all(&app_dir)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to create app data directory: {}", e)))?;

    let db_path = app_dir.join("app_state.db");
    let conn = Connection::open(&db_path)?;

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

    // Migrations for columns added after initial schema
    conn.execute("ALTER TABLE promoted_items ADD COLUMN sub_folder TEXT", []).ok();

    let is_first_run: rusqlite::Result<String> = conn.query_row(
        "SELECT value FROM settings WHERE key = 'initialized_defaults'",
        [],
        |row| row.get(0),
    );

    if is_first_run.is_err() {
        conn.execute_batch("
            INSERT OR IGNORE INTO agents (id, name, skills_path, rules_path) VALUES
                ('cursor', 'Cursor', '.cursor/skills', '.cursor/rules'),
                ('jetbrains', 'JetBrains AI', '.agents/skills', '.agents/rules'),
                ('claude', 'Claude Code', '.claude/skills', '.claude/rules');

            INSERT OR IGNORE INTO settings (key, value) VALUES ('initialized_defaults', 'true');
        ")?;
    }

    Ok(conn)
}
