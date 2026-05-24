
# Rust: Use spawn_blocking for Blocking I/O

Tauri's async runtime is a Tokio executor. Calling blocking operations — `fs::*`, `Command::new(...)`, `fs::read_dir`, heavy computation — directly inside an `async` command stalls the executor and can starve other tasks.

Wrap all blocking work in `tauri::async_runtime::spawn_blocking` and await the result.

**Wrong:**
```rust
#[tauri::command]
pub async fn sync_repo(repo_url: String) -> Result<(), String> {
    fs::create_dir_all(&repo_dir)?;          // blocks
    Command::new("git").arg("clone")...output()?;  // blocks
}
```

**Right:**
```rust
#[tauri::command]
pub async fn sync_repo(repo_url: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        fs::create_dir_all(&repo_dir).map_err(|e| e.to_string())?;
        Command::new("git").arg("clone")...output().map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}
```

Network calls via `reqwest` are already async — they do not need `spawn_blocking`.
