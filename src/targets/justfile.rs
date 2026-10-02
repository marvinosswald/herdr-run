use super::{runnable, RunnableTarget};
use crate::runnable::Runnable;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub struct Justfile;

impl RunnableTarget for Justfile {
    fn kind(&self) -> &'static str {
        "justfile"
    }

    fn file_names(&self) -> &'static [&'static str] {
        &["justfile", "Justfile", ".justfile"]
    }

    fn extract(&self, path: &Path) -> Result<Vec<Runnable>> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;

        let kind = self.kind();
        let mut runnables = Vec::new();

        for line in raw.lines() {
            let mut trimmed = line.trim_start();
            trimmed = trimmed.strip_prefix('@').unwrap_or(trimmed).trim_start();
            if trimmed.is_empty()
                || trimmed.starts_with('#')
                || trimmed.starts_with("set ")
                || trimmed.starts_with("mod ")
                || trimmed.starts_with("import ")
                || trimmed.contains(":=")
            {
                continue;
            }

            if let Some((recipe, _)) = trimmed.split_once(':') {
                let recipe = recipe.split_whitespace().next().unwrap_or("");
                if recipe.is_empty()
                    || recipe.starts_with('_')
                    || recipe.starts_with("alias ")
                    || recipe.contains(|c: char| {
                        !c.is_ascii_alphanumeric() && c != '_' && c != '-' && c != '.'
                    })
                {
                    continue;
                }
                runnables.push(runnable(path, kind, recipe, format!("just {recipe}")));
            }
        }

        Ok(runnables)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn extracts_just_recipes() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("justfile");
        fs::write(
            &path,
            r#"
set shell := ["bash", "-c"]

build:
    echo build

@test:
    echo test

_private:
    echo ignored

alias b := build
"#,
        )
        .unwrap();

        let runnables = Justfile.extract(&path).unwrap();
        let names: Vec<_> = runnables.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"build"));
        assert!(names.contains(&"test"));
        assert!(!names.contains(&"_private"));
        assert!(!names.contains(&"alias"));
        assert!(runnables
            .iter()
            .any(|r| r.name == "build" && r.command == "just build"));
    }
}
