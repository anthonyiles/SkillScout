use crate::db::AppState;
use crate::models::{ItemSelection, PromotedItem, RepositoryItem};
use rusqlite::{params, Connection};
use tauri::{command, State};

fn row_to_repo_item(row: &rusqlite::Row) -> rusqlite::Result<RepositoryItem> {
    Ok(RepositoryItem {
        id: row.get(0)?,
        name: row.get(1)?,
        folder: row.get(2)?,
        description: row.get(3)?,
        file_path: row.get(4)?,
        content: row.get(5)?,
        sha: row.get(6)?,
        last_synced: row.get(7)?,
    })
}

// ── Inner DB helpers (pub(crate) so tests can call them directly) ───────────

pub(crate) fn db_get_repository_items(conn: &Connection, folder: Option<&str>) -> Result<Vec<RepositoryItem>, String> {
    let base_query = "SELECT id, name, folder, description, file_path, content, sha, last_synced FROM repository_items";
    let mut items = Vec::new();

    if let Some(folder_name) = folder {
        let query = format!("{} WHERE folder = ?1", base_query);
        let mut stmt = conn.prepare(&query).map_err(|e| { eprintln!("Failed to prepare items query: {}", e); "Failed to fetch items".to_string() })?;
        let iter = stmt.query_map(params![folder_name], row_to_repo_item).map_err(|e| { eprintln!("Failed to query items: {}", e); "Failed to fetch items".to_string() })?;
        for item in iter { items.push(item.map_err(|e| { eprintln!("Corrupt item row: {}", e); "Corrupt data in database".to_string() })?); }
    } else {
        let mut stmt = conn.prepare(base_query).map_err(|e| { eprintln!("Failed to prepare items query: {}", e); "Failed to fetch items".to_string() })?;
        let iter = stmt.query_map([], row_to_repo_item).map_err(|e| { eprintln!("Failed to query items: {}", e); "Failed to fetch items".to_string() })?;
        for item in iter { items.push(item.map_err(|e| { eprintln!("Corrupt item row: {}", e); "Corrupt data in database".to_string() })?); }
    }

    Ok(items)
}

pub(crate) fn db_get_item_selections(conn: &Connection) -> Result<Vec<ItemSelection>, String> {
    let mut stmt = conn.prepare("SELECT item_id, project_id, applied_sha FROM item_selections")
        .map_err(|e| { eprintln!("Failed to prepare selections query: {}", e); "Failed to fetch selections".to_string() })?;

    let iter = stmt.query_map([], |row| {
        Ok(ItemSelection {
            item_id: row.get(0)?,
            project_id: row.get(1)?,
            applied_sha: row.get(2)?,
        })
    }).map_err(|e| { eprintln!("Failed to query selections: {}", e); "Failed to fetch selections".to_string() })?;

    let mut result = Vec::new();
    for selection in iter { result.push(selection.map_err(|e| { eprintln!("Corrupt selection row: {}", e); "Corrupt selection data".to_string() })?); }
    Ok(result)
}

pub(crate) fn db_toggle_item_selection(conn: &Connection, item_id: &str, project_id: i64) -> Result<(), String> {
    let deleted = conn.execute(
        "DELETE FROM item_selections WHERE item_id = ?1 AND project_id = ?2",
        params![item_id, project_id],
    ).map_err(|e| { eprintln!("Failed to delete selection: {}", e); "Failed to update selection".to_string() })?;

    if deleted == 0 {
        conn.execute(
            "INSERT INTO item_selections (item_id, project_id) VALUES (?1, ?2)",
            params![item_id, project_id],
        ).map_err(|e| { eprintln!("Failed to insert selection: {}", e); "Failed to update selection".to_string() })?;
    }

    Ok(())
}

pub(crate) fn db_update_applied_sha(conn: &Connection, item_id: &str, project_id: i64, sha: &str) -> Result<(), String> {
    conn.execute(
        "UPDATE item_selections SET applied_sha = ?1 WHERE item_id = ?2 AND project_id = ?3",
        params![sha, item_id, project_id],
    ).map_err(|e| { eprintln!("Failed to update applied sha: {}", e); "Failed to update selection status".to_string() })?;
    Ok(())
}

pub(crate) fn db_get_promoted_items(conn: &Connection) -> Result<Vec<PromotedItem>, String> {
    let mut stmt = conn.prepare("SELECT id, name, path, item_type, repository_item_id, url, branch, sub_folder FROM promoted_items")
        .map_err(|e| { eprintln!("Failed to prepare promoted items query: {}", e); "Failed to fetch promoted items".to_string() })?;

    let iter = stmt.query_map([], |row| {
        Ok(PromotedItem {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            path: row.get(2)?,
            item_type: row.get(3)?,
            repository_item_id: row.get(4)?,
            url: row.get(5)?,
            branch: row.get(6)?,
            sub_folder: row.get(7)?,
        })
    }).map_err(|e| { eprintln!("Failed to query promoted items: {}", e); "Failed to fetch promoted items".to_string() })?;

    let mut result = Vec::new();
    for promoted in iter { result.push(promoted.map_err(|e| { eprintln!("Corrupt promoted item row: {}", e); "Corrupt promoted item data".to_string() })?); }
    Ok(result)
}

pub(crate) fn db_add_promoted_item(conn: &Connection, item: PromotedItem) -> Result<PromotedItem, String> {
    conn.execute(
        "INSERT INTO promoted_items (name, path, item_type, repository_item_id, url, branch, sub_folder) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![item.name, item.path, item.item_type, item.repository_item_id, item.url, item.branch, item.sub_folder],
    ).map_err(|e| { eprintln!("Failed to insert promoted item: {}", e); "Failed to save promoted item".to_string() })?;

    let mut saved = item;
    saved.id = Some(conn.last_insert_rowid());
    Ok(saved)
}

pub(crate) fn db_remove_promoted_item(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM promoted_items WHERE id = ?1", params![id])
        .map_err(|e| { eprintln!("Failed to remove promoted item: {}", e); "Failed to remove promoted item".to_string() })?;
    Ok(())
}

// ── Tauri commands (thin wrappers over DB helpers) ───────────────────────────

#[command]
pub fn get_repository_items(state: State<'_, AppState>, folder: Option<String>) -> Result<Vec<RepositoryItem>, String> {
    db_get_repository_items(&state.lock_conn(), folder.as_deref())
}

#[command]
pub fn get_item_selections(state: State<'_, AppState>) -> Result<Vec<ItemSelection>, String> {
    db_get_item_selections(&state.lock_conn())
}

#[command]
pub fn toggle_item_selection(state: State<'_, AppState>, item_id: String, project_id: i64) -> Result<(), String> {
    db_toggle_item_selection(&state.lock_conn(), &item_id, project_id)
}

#[command]
pub fn update_applied_sha(state: State<'_, AppState>, item_id: String, project_id: i64, sha: String) -> Result<(), String> {
    db_update_applied_sha(&state.lock_conn(), &item_id, project_id, &sha)
}

#[command]
pub fn get_promoted_items(state: State<'_, AppState>) -> Result<Vec<PromotedItem>, String> {
    db_get_promoted_items(&state.lock_conn())
}

#[command]
pub fn add_promoted_item(state: State<'_, AppState>, item: PromotedItem) -> Result<PromotedItem, String> {
    db_add_promoted_item(&state.lock_conn(), item)
}

#[command]
pub fn remove_promoted_item(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    db_remove_promoted_item(&state.lock_conn(), id)
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

    fn insert_test_item(conn: &Connection, id: &str, folder: &str) {
        conn.execute(
            "INSERT INTO repository_items (id, name, folder, file_path, content, sha) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, format!("{}.md", id), folder, format!("/{}/{}.md", folder, id), "# Content", "abc123"],
        ).unwrap();
    }

    fn insert_test_project(conn: &Connection, path: &str) -> i64 {
        conn.execute("INSERT INTO projects (path) VALUES (?1)", params![path]).unwrap();
        conn.last_insert_rowid()
    }

    // ── Repository items ─────────────────────────────────────────────────────

    #[test]
    fn get_repository_items_returns_empty_when_no_items() {
        let conn = open_test_db();
        let items = db_get_repository_items(&conn, None).unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn get_repository_items_returns_all_items() {
        let conn = open_test_db();
        insert_test_item(&conn, "skill-a", "skills");
        insert_test_item(&conn, "rule-b", "rules");

        let items = db_get_repository_items(&conn, None).unwrap();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn get_repository_items_filters_by_folder() {
        let conn = open_test_db();
        insert_test_item(&conn, "skill-a", "skills");
        insert_test_item(&conn, "rule-b", "rules");

        let skills = db_get_repository_items(&conn, Some("skills")).unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].folder, "skills");
    }

    #[test]
    fn get_repository_items_folder_filter_returns_empty_for_unknown_folder() {
        let conn = open_test_db();
        insert_test_item(&conn, "skill-a", "skills");

        let items = db_get_repository_items(&conn, Some("nonexistent")).unwrap();
        assert!(items.is_empty());
    }

    // ── Item selections ──────────────────────────────────────────────────────

    #[test]
    fn get_item_selections_returns_empty_initially() {
        let conn = open_test_db();
        let selections = db_get_item_selections(&conn).unwrap();
        assert!(selections.is_empty());
    }

    #[test]
    fn toggle_item_selection_inserts_on_first_call() {
        let conn = open_test_db();
        insert_test_item(&conn, "skill-a", "skills");
        let project_id = insert_test_project(&conn, "/home/user/alpha");

        db_toggle_item_selection(&conn, "skill-a", project_id).unwrap();

        let selections = db_get_item_selections(&conn).unwrap();
        assert_eq!(selections.len(), 1);
        assert_eq!(selections[0].item_id, "skill-a");
        assert_eq!(selections[0].project_id, project_id);
    }

    #[test]
    fn toggle_item_selection_removes_on_second_call() {
        let conn = open_test_db();
        insert_test_item(&conn, "skill-a", "skills");
        let project_id = insert_test_project(&conn, "/home/user/beta");

        db_toggle_item_selection(&conn, "skill-a", project_id).unwrap();
        db_toggle_item_selection(&conn, "skill-a", project_id).unwrap();

        let selections = db_get_item_selections(&conn).unwrap();
        assert!(selections.is_empty());
    }

    #[test]
    fn update_applied_sha_sets_sha_on_existing_selection() {
        let conn = open_test_db();
        insert_test_item(&conn, "skill-a", "skills");
        let project_id = insert_test_project(&conn, "/home/user/gamma");

        db_toggle_item_selection(&conn, "skill-a", project_id).unwrap();
        db_update_applied_sha(&conn, "skill-a", project_id, "newsha123").unwrap();

        let selections = db_get_item_selections(&conn).unwrap();
        assert_eq!(selections[0].applied_sha.as_deref(), Some("newsha123"));
    }

    #[test]
    fn applied_sha_starts_as_none() {
        let conn = open_test_db();
        insert_test_item(&conn, "skill-b", "skills");
        let project_id = insert_test_project(&conn, "/home/user/delta");

        db_toggle_item_selection(&conn, "skill-b", project_id).unwrap();

        let selections = db_get_item_selections(&conn).unwrap();
        assert!(selections[0].applied_sha.is_none());
    }

    // ── Promoted items ───────────────────────────────────────────────────────

    #[test]
    fn get_promoted_items_returns_empty_initially() {
        let conn = open_test_db();
        let items = db_get_promoted_items(&conn).unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn add_promoted_item_assigns_id() {
        let conn = open_test_db();
        let item = PromotedItem {
            id: None,
            name: "my-skill.md".to_string(),
            path: "/path/to/skill".to_string(),
            item_type: "skills".to_string(),
            repository_item_id: None,
            url: Some("https://github.com/org/repo/pull/1".to_string()),
            branch: "feat/my-skill".to_string(),
            sub_folder: None,
        };
        let saved = db_add_promoted_item(&conn, item).unwrap();
        assert!(saved.id.is_some());
    }

    #[test]
    fn add_promoted_item_is_retrievable() {
        let conn = open_test_db();
        let item = PromotedItem {
            id: None,
            name: "my-rule.md".to_string(),
            path: "/path/to/rule".to_string(),
            item_type: "rules".to_string(),
            repository_item_id: None,
            url: None,
            branch: "feat/my-rule".to_string(),
            sub_folder: Some("custom".to_string()),
        };
        db_add_promoted_item(&conn, item).unwrap();

        let items = db_get_promoted_items(&conn).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "my-rule.md");
        assert_eq!(items[0].sub_folder.as_deref(), Some("custom"));
    }

    #[test]
    fn remove_promoted_item_deletes_it() {
        let conn = open_test_db();
        let item = PromotedItem {
            id: None,
            name: "gone.md".to_string(),
            path: "/path/gone".to_string(),
            item_type: "skills".to_string(),
            repository_item_id: None,
            url: None,
            branch: "feat/gone".to_string(),
            sub_folder: None,
        };
        let saved = db_add_promoted_item(&conn, item).unwrap();
        db_remove_promoted_item(&conn, saved.id.unwrap()).unwrap();

        let items = db_get_promoted_items(&conn).unwrap();
        assert!(items.is_empty());
    }
}
