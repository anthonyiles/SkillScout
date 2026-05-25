
# Rust: Validate User-Supplied Paths

Never join a user-supplied filename or path segment directly into a filesystem path without validation. A path containing `..` or an absolute component can escape the intended directory.

At every IPC → filesystem boundary, validate that the resulting path stays within the intended base directory.

**Wrong:**
```rust
// task.file_name comes from the frontend — could be "../../etc/passwd"
let target = base_dir.join(&task.file_name);
fs::copy(&src, &target)?;
```

**Right:**
```rust
use std::path::{Component, Path};

fn safe_join(base: &Path, user_input: &str) -> Result<PathBuf, String> {
    let input = Path::new(user_input);
    for component in input.components() {
        match component {
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(format!("Invalid path component in: {}", user_input));
            }
            _ => {}
        }
    }
    let result = base.join(input);
    // Belt-and-suspenders: canonicalize and assert prefix
    let canonical_base = base.canonicalize().map_err(|e| e.to_string())?;
    let canonical_result = result.canonicalize().map_err(|e| e.to_string())?;
    if !canonical_result.starts_with(&canonical_base) {
        return Err(format!("Path escapes base directory: {}", user_input));
    }
    Ok(result)
}
```

This applies to: `check_existing`, `apply_skills`, `get_project_files`, and any command that takes a filename from the frontend and uses it on the filesystem.
