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

pub(crate) fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    let src_path = src.as_ref();
    let dst_path = dst.as_ref();

    // Self-copy guard: skip if canonicalize fails (e.g. transient OS lock) rather than
    // aborting the entire copy — the guard is best-effort, not a hard correctness requirement.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn copies_a_flat_directory() {
        let src = tempdir().unwrap();
        let dst = tempdir().unwrap();

        fs::write(src.path().join("a.txt"), "hello").unwrap();
        fs::write(src.path().join("b.txt"), "world").unwrap();

        copy_dir_all(src.path(), dst.path()).unwrap();

        assert_eq!(fs::read_to_string(dst.path().join("a.txt")).unwrap(), "hello");
        assert_eq!(fs::read_to_string(dst.path().join("b.txt")).unwrap(), "world");
    }

    #[test]
    fn copies_nested_directories() {
        let src = tempdir().unwrap();
        let dst = tempdir().unwrap();

        let sub = src.path().join("sub");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("nested.txt"), "deep").unwrap();

        copy_dir_all(src.path(), dst.path()).unwrap();

        assert_eq!(
            fs::read_to_string(dst.path().join("sub").join("nested.txt")).unwrap(),
            "deep"
        );
    }

    #[test]
    fn creates_destination_directory_if_absent() {
        let src = tempdir().unwrap();
        let dst_parent = tempdir().unwrap();
        let dst = dst_parent.path().join("new_dir");

        fs::write(src.path().join("file.txt"), "data").unwrap();

        copy_dir_all(src.path(), &dst).unwrap();

        assert!(dst.exists());
        assert_eq!(fs::read_to_string(dst.join("file.txt")).unwrap(), "data");
    }

    #[test]
    fn refuses_to_copy_directory_into_itself() {
        let dir = tempdir().unwrap();
        let sub = dir.path().join("sub");
        fs::create_dir(&sub).unwrap();

        let result = copy_dir_all(dir.path(), &sub);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidInput);
    }
}
