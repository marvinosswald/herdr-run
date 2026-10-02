use crate::context;
use crate::runnable::Runnable;
use crate::targets::RunnableTarget;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub fn scan(start: &Path, targets: &[Box<dyn RunnableTarget>]) -> Result<(Vec<Runnable>, PathBuf)> {
    let start = start.canonicalize()?;
    let root = context::git_root(&start).unwrap_or_else(|| start.clone());

    let mut runnables = Vec::new();
    let mut dir = Some(start.clone());

    while let Some(current) = dir {
        for target in targets {
            for file_name in target.file_names() {
                let path = current.join(file_name);
                if path.is_file() {
                    match target.extract(&path) {
                        Ok(mut found) => runnables.append(&mut found),
                        Err(e) => eprintln!("warning: failed to read {}: {e}", path.display()),
                    }
                }
            }
        }

        if current == root {
            break;
        }
        dir = current.parent().map(Path::to_path_buf);
    }

    Ok((runnables, root))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry;
    use std::fs;
    use tempfile::TempDir;

    fn init_git(dir: &Path) {
        let output = std::process::Command::new("git")
            .args(["init", dir.to_str().unwrap_or(".")])
            .output()
            .expect("git init failed");
        assert!(output.status.success());
    }

    #[test]
    fn scan_finds_runnables_in_start_directory() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{"scripts":{"test":"jest"}}"#,
        )
        .unwrap();

        let (runnables, root) = scan(dir.path(), &registry::default_targets()).unwrap();

        assert_eq!(root, dir.path().canonicalize().unwrap());
        assert_eq!(runnables.len(), 1);
        assert_eq!(runnables[0].name, "test");
    }

    #[test]
    fn scan_walks_up_to_git_root() {
        let dir = TempDir::new().unwrap();
        init_git(dir.path());
        let sub = dir.path().join("sub");
        fs::create_dir(&sub).unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{"scripts":{"root":"echo root"}}"#,
        )
        .unwrap();
        fs::write(sub.join("justfile"), "build:\n    echo build\n").unwrap();

        let (runnables, root) = scan(&sub, &registry::default_targets()).unwrap();

        assert_eq!(root, dir.path().canonicalize().unwrap());
        let names: Vec<_> = runnables.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"root"));
        assert!(names.contains(&"build"));
    }

    #[test]
    fn scan_returns_empty_when_no_runnables() {
        let dir = TempDir::new().unwrap();

        let (runnables, root) = scan(dir.path(), &registry::default_targets()).unwrap();

        assert!(runnables.is_empty());
        assert_eq!(root, dir.path().canonicalize().unwrap());
    }
}
