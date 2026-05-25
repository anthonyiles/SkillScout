use crate::db::AppState;
use crate::models::{ItemSelection, PromotedItem, RepositoryItem};
use rusqlite::params;
use tauri::{command, State};

#[command]
pub fn get_repository_items(state: State<'_, AppState>, folder: Option<String>) -> Result<Vec<RepositoryItem>, String> {
    let conn = state.lock_conn();

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

#[command]
pub fn get_item_selections(state: State<'_, AppState>) -> Result<Vec<ItemSelection>, String> {
    let conn = state.lock_conn();
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

#[command]
pub fn toggle_item_selection(state: State<'_, AppState>, item_id: String, project_id: i64) -> Result<(), String> {
    let conn = state.lock_conn();

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

#[command]
pub fn update_applied_sha(state: State<'_, AppState>, item_id: String, project_id: i64, sha: String) -> Result<(), String> {
    let conn = state.lock_conn();
    conn.execute(
        "UPDATE item_selections SET applied_sha = ?1 WHERE item_id = ?2 AND project_id = ?3",
        params![sha, item_id, project_id],
    ).map_err(|e| { eprintln!("Failed to update applied sha: {}", e); "Failed to update selection status".to_string() })?;
    Ok(())
}

#[command]
pub fn get_promoted_items(state: State<'_, AppState>) -> Result<Vec<PromotedItem>, String> {
    let conn = state.lock_conn();
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

#[command]
pub fn add_promoted_item(state: State<'_, AppState>, item: PromotedItem) -> Result<PromotedItem, String> {
    let conn = state.lock_conn();
    conn.execute(
        "INSERT INTO promoted_items (name, path, item_type, repository_item_id, url, branch, sub_folder) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![item.name, item.path, item.item_type, item.repository_item_id, item.url, item.branch, item.sub_folder],
    ).map_err(|e| { eprintln!("Failed to insert promoted item: {}", e); "Failed to save promoted item".to_string() })?;

    let mut saved = item;
    saved.id = Some(conn.last_insert_rowid());
    Ok(saved)
}

#[command]
pub fn remove_promoted_item(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let conn = state.lock_conn();
    conn.execute("DELETE FROM promoted_items WHERE id = ?1", params![id])
        .map_err(|e| { eprintln!("Failed to remove promoted item: {}", e); "Failed to remove promoted item".to_string() })?;
    Ok(())
}
