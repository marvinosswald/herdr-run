use super::{runnable, RunnableTarget};
use crate::runnable::Runnable;
use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::path::Path;

pub struct PackageJson;

impl RunnableTarget for PackageJson {
    fn kind(&self) -> &'static str {
        "package.json"
    }

    fn file_names(&self) -> &'static [&'static str] {
        &["package.json"]
    }

    fn extract(&self, path: &Path) -> Result<Vec<Runnable>> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let value: Value = serde_json::from_str(&raw)
            .with_context(|| format!("failed to parse {}", path.display()))?;

        let scripts = value
            .get("scripts")
            .and_then(|v| v.as_object())
            .context("package.json has no 'scripts' object")?;

        let pm = detect_package_manager(path.parent().unwrap_or(Path::new(".")));
        let kind = self.kind();

        let mut runnables = Vec::new();
        for (name, _) in scripts {
            runnables.push(runnable(path, kind, name, format!("{pm} {name}")));
        }

        Ok(runnables)
    }
}

fn detect_package_manager(dir: &Path) -> &'static str {
    for name in ["yarn.lock", "pnpm-lock.yaml", "bun.lockb", "bun.lock"] {
        if dir.join(name).exists() {
            return match name {
                "yarn.lock" => "yarn",
                "pnpm-lock.yaml" => "pnpm",
                _ => "bun",
            };
        }
    }
    "npm run"
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn write_package_json(dir: &Path, content: &str) -> PathBuf {
        let path = dir.join("package.json");
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn extracts_npm_scripts() {
        let dir = TempDir::new().unwrap();
        let path = write_package_json(dir.path(), r#"{"scripts":{"build":"tsc","test":"jest"}}"#);

        let runnables = PackageJson.extract(&path).unwrap();
        let names: Vec<_> = runnables.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"build"));
        assert!(names.contains(&"test"));
        assert!(runnables
            .iter()
            .any(|r| r.name == "build" && r.command == "npm run build"));
        assert!(runnables
            .iter()
            .any(|r| r.name == "test" && r.command == "npm run test"));
    }

    #[test]
    fn detects_yarn_lock() {
        let dir = TempDir::new().unwrap();
        let path = write_package_json(dir.path(), r#"{"scripts":{"dev":"next"}}"#);
        fs::write(dir.path().join("yarn.lock"), "").unwrap();

        let runnables = PackageJson.extract(&path).unwrap();
        assert_eq!(runnables.len(), 1);
        assert_eq!(runnables[0].command, "yarn dev");
    }

    #[test]
    fn detects_pnpm_lock() {
        let dir = TempDir::new().unwrap();
        let path = write_package_json(dir.path(), r#"{"scripts":{"dev":"next"}}"#);
        fs::write(dir.path().join("pnpm-lock.yaml"), "").unwrap();

        let runnables = PackageJson.extract(&path).unwrap();
        assert_eq!(runnables[0].command, "pnpm dev");
    }

    #[test]
    fn detects_bun_lock() {
        let dir = TempDir::new().unwrap();
        let path = write_package_json(dir.path(), r#"{"scripts":{"dev":"next"}}"#);
        fs::write(dir.path().join("bun.lockb"), "").unwrap();

        let runnables = PackageJson.extract(&path).unwrap();
        assert_eq!(runnables[0].command, "bun dev");
    }

    #[test]
    fn missing_scripts_returns_error() {
        let dir = TempDir::new().unwrap();
        let path = write_package_json(dir.path(), r#"{"name":"foo"}"#);

        assert!(PackageJson.extract(&path).is_err());
    }
}
