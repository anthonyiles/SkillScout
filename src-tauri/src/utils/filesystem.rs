use std::fs;
use std::path::{Path, PathBuf};

fn resolve_path(path: &Path) -> PathBuf {
    let mut ancestor = path;
    let mut suffix = PathBuf::new();
    while !ancestor.exists() {
        if let Some(parent) = ancestor.parent() {
            if let Some(name) = ancestor.file_name() {
                let mut new_suffix = PathBuf::from(name);
                new_suffix.push(suffix);
                suffix = new_suffix;
            }
            ancestor = parent;
        } else {
            break;
        }
    }
    if let Ok(canon) = fs::canonicalize(ancestor) {
        canon.join(suffix)
    } else {
        path.to_path_buf()
    }
}

pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    let src_path = src.as_ref();
    let dst_path = dst.as_ref();

    if let Ok(src_canon) = fs::canonicalize(src_path) {
        let dst_canon = resolve_path(dst_path);
        if dst_canon.starts_with(&src_canon) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Cannot copy a directory into itself",
            ));
        }
    }

    fs::create_dir_all(dst_path)?;
    for entry in fs::read_dir(src_path)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let path = entry.path();
        let next_dst = dst_path.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&path, &next_dst)?;
        } else {
            fs::copy(&path, &next_dst)?;
        }
    }
    Ok(())
}
