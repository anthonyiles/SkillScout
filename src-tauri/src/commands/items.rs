use crate::db::AppState;
use crate::models::{ItemSelection, PromotedItem, RepositoryItem};
use rusqlite::params;
use tauri::{command, State};

#[command]
pub fn get_repository_items(state: State<'_, AppState>, folder: Option<String>) -> Result<Vec<RepositoryItem>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    
    let mut query = String::from("SELECT id, name, folder, description, file_path, content, sha, last_synced FROM repository_items");
    let mut items = Vec::new();

    if let Some(f) = folder {
        query.push_str(" WHERE folder = ?1");
        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
        let items_iter = stmt.query_map(params![f], row_to_repo_item).map_err(|e| e.to_string())?;
        for i in items_iter { items.push(i.map_err(|e| e.to_string())?); }
    } else {
        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
        let items_iter = stmt.query_map([], row_to_repo_item).map_err(|e| e.to_string())?;
        for i in items_iter { items.push(i.map_err(|e| e.to_string())?); }
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
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT item_id, project_id, applied_sha FROM item_selections").map_err(|e| e.to_string())?;
    
    let iter = stmt.query_map([], |row| {
        Ok(ItemSelection {
            item_id: row.get(0)?,
            project_id: row.get(1)?,
            applied_sha: row.get(2)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for i in iter { result.push(i.map_err(|e| e.to_string())?); }
    Ok(result)
}

#[command]
pub fn toggle_item_selection(state: State<'_, AppState>, item_id: String, project_id: i64) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    
    // Try to delete first; if no rows affected, insert
    let deleted = conn.execute(
        "DELETE FROM item_selections WHERE item_id = ?1 AND project_id = ?2",
        params![item_id, project_id]
    ).map_err(|e| e.to_string())?;
    
    if deleted == 0 {
        conn.execute(
            "INSERT INTO item_selections (item_id, project_id) VALUES (?1, ?2)",
            params![item_id, project_id]
        ).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[command]
pub fn update_applied_sha(state: State<'_, AppState>, item_id: String, project_id: i64, sha: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE item_selections SET applied_sha = ?1 WHERE item_id = ?2 AND project_id = ?3",
        params![sha, item_id, project_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
pub fn get_promoted_items(state: State<'_, AppState>) -> Result<Vec<PromotedItem>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, name, path, item_type, repository_item_id, url, branch FROM promoted_items").map_err(|e| e.to_string())?;
    
    let iter = stmt.query_map([], |row| {
        Ok(PromotedItem {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            path: row.get(2)?,
            item_type: row.get(3)?,
            repository_item_id: row.get(4)?,
            url: row.get(5)?,
            branch: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for i in iter { result.push(i.map_err(|e| e.to_string())?); }
    Ok(result)
}

#[command]
pub fn add_promoted_item(state: State<'_, AppState>, item: PromotedItem) -> Result<PromotedItem, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO promoted_items (name, path, item_type, repository_item_id, url, branch) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![item.name, item.path, item.item_type, item.repository_item_id, item.url, item.branch],
    ).map_err(|e| e.to_string())?;
    
    let mut saved = item;
    saved.id = Some(conn.last_insert_rowid());
    Ok(saved)
}

#[command]
pub fn remove_promoted_item(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM promoted_items WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
    Ok(())
}
