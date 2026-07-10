use std::path::{Component, Path, PathBuf};

/// Resolves a scenario-relative path against `work_dir`, rejecting absolute
/// paths and `..` components so a spec can never accidentally (or
/// maliciously) write outside its own temp directory.
pub fn resolve_safe(work_dir: &Path, path: &str) -> PathBuf {
    let relative = Path::new(path);
    assert!(
        !relative.is_absolute() && !relative.components().any(|c| c == Component::ParentDir),
        "Unsafe scenario path: {path}"
    );
    work_dir.join(relative)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_relative_paths_under_work_dir() {
        let work_dir = Path::new("/tmp/work");
        assert_eq!(
            resolve_safe(work_dir, "a/b.txt"),
            PathBuf::from("/tmp/work/a/b.txt")
        );
    }

    #[test]
    #[should_panic(expected = "Unsafe scenario path")]
    fn rejects_absolute_paths() {
        resolve_safe(Path::new("/tmp/work"), "/etc/passwd");
    }

    #[test]
    #[should_panic(expected = "Unsafe scenario path")]
    fn rejects_parent_dir_components() {
        resolve_safe(Path::new("/tmp/work"), "../escape.txt");
    }
}
