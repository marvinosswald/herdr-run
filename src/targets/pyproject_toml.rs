use super::{runnable, RunnableTarget};
use crate::runnable::Runnable;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub struct PyprojectToml;

impl RunnableTarget for PyprojectToml {
    fn kind(&self) -> &'static str {
        "pyproject.toml"
    }

    fn file_names(&self) -> &'static [&'static str] {
        &["pyproject.toml"]
    }

    fn extract(&self, path: &Path) -> Result<Vec<Runnable>> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let value: toml::Value =
            toml::from_str(&raw).with_context(|| format!("failed to parse {}", path.display()))?;

        let kind = self.kind();
        let mut runnables = Vec::new();

        if let Some(tasks) = value
            .get("tool")
            .and_then(|t| t.get("poe"))
            .and_then(|p| p.get("tasks"))
            .and_then(|v| v.as_table())
        {
            for name in tasks.keys() {
                runnables.push(runnable(path, kind, name, format!("poe {name}")));
            }
        }

        if let Some(tasks) = value
            .get("tool")
            .and_then(|t| t.get("taskipy"))
            .and_then(|p| p.get("tasks"))
            .and_then(|v| v.as_table())
        {
            for name in tasks.keys() {
                runnables.push(runnable(path, kind, name, format!("task {name}")));
            }
        }

        if let Some(scripts) = value
            .get("tool")
            .and_then(|t| t.get("poetry"))
            .and_then(|p| p.get("scripts"))
            .and_then(|v| v.as_table())
        {
            for name in scripts.keys() {
                runnables.push(runnable(path, kind, name, format!("poetry run {name}")));
            }
        }

        Ok(runnables)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn write_pyproject(dir: &Path, content: &str) -> PathBuf {
        let path = dir.join("pyproject.toml");
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn extracts_poe_tasks() {
        let dir = TempDir::new().unwrap();
        let path = write_pyproject(
            dir.path(),
            r#"
[tool.poe.tasks]
test = "pytest"
lint = "ruff check ."
"#,
        );

        let runnables = PyprojectToml.extract(&path).unwrap();
        assert!(runnables
            .iter()
            .any(|r| r.name == "test" && r.command == "poe test"));
        assert!(runnables
            .iter()
            .any(|r| r.name == "lint" && r.command == "poe lint"));
    }

    #[test]
    fn extracts_taskipy_tasks() {
        let dir = TempDir::new().unwrap();
        let path = write_pyproject(
            dir.path(),
            r#"
[tool.taskipy.tasks]
test = "pytest"
"#,
        );

        let runnables = PyprojectToml.extract(&path).unwrap();
        assert!(runnables
            .iter()
            .any(|r| r.name == "test" && r.command == "task test"));
    }

    #[test]
    fn extracts_poetry_scripts() {
        let dir = TempDir::new().unwrap();
        let path = write_pyproject(
            dir.path(),
            r#"
[tool.poetry.scripts]
serve = "app:serve"
"#,
        );

        let runnables = PyprojectToml.extract(&path).unwrap();
        assert!(runnables
            .iter()
            .any(|r| r.name == "serve" && r.command == "poetry run serve"));
    }
}
