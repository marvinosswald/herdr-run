use super::{runnable, RunnableTarget};
use crate::runnable::Runnable;
use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::path::Path;

pub struct ComposerJson;

impl RunnableTarget for ComposerJson {
    fn kind(&self) -> &'static str {
        "composer.json"
    }

    fn file_names(&self) -> &'static [&'static str] {
        &["composer.json"]
    }

    fn extract(&self, path: &Path) -> Result<Vec<Runnable>> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let value: Value = serde_json::from_str(&raw)
            .with_context(|| format!("failed to parse {}", path.display()))?;

        let scripts = value
            .get("scripts")
            .and_then(|v| v.as_object())
            .context("composer.json has no 'scripts' object")?;

        let kind = self.kind();
        let mut runnables = Vec::new();
        for name in scripts.keys() {
            runnables.push(runnable(
                path,
                kind,
                name,
                format!("composer run-script {name}"),
            ));
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
    fn extracts_composer_scripts() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("composer.json");
        fs::write(&path, r#"{"scripts":{"test":"phpunit","cs":"phpcs"}}"#).unwrap();

        let runnables = ComposerJson.extract(&path).unwrap();
        let names: Vec<_> = runnables.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"test"));
        assert!(names.contains(&"cs"));
        assert!(runnables
            .iter()
            .any(|r| r.name == "test" && r.command == "composer run-script test"));
    }

    #[test]
    fn missing_scripts_returns_error() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("composer.json");
        fs::write(&path, r#"{"name":"foo/bar"}"#).unwrap();

        assert!(ComposerJson.extract(&path).is_err());
    }
}
