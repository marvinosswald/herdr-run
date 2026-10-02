use super::{runnable, RunnableTarget};
use crate::runnable::Runnable;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub struct CargoToml;

impl RunnableTarget for CargoToml {
    fn kind(&self) -> &'static str {
        "Cargo.toml"
    }

    fn file_names(&self) -> &'static [&'static str] {
        &["Cargo.toml"]
    }

    fn extract(&self, path: &Path) -> Result<Vec<Runnable>> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let value: toml::Value =
            toml::from_str(&raw).with_context(|| format!("failed to parse {}", path.display()))?;

        let kind = self.kind();
        let mut runnables = vec![
            runnable(path, kind, "run", "cargo run"),
            runnable(path, kind, "test", "cargo test"),
            runnable(path, kind, "build", "cargo build"),
            runnable(path, kind, "check", "cargo check"),
            runnable(path, kind, "clippy", "cargo clippy"),
        ];

        if let Some(bins) = value.get("bin").and_then(|v| v.as_array()) {
            for bin in bins {
                if let Some(name) = bin.get("name").and_then(|v| v.as_str()) {
                    runnables.push(runnable(
                        path,
                        kind,
                        format!("run:{name}"),
                        format!("cargo run --bin {name}"),
                    ));
                }
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
    fn extracts_default_cargo_runnables() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("Cargo.toml");
        fs::write(
            &path,
            r#"
[package]
name = "example"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();

        let runnables = CargoToml.extract(&path).unwrap();
        let names: Vec<_> = runnables.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"run"));
        assert!(names.contains(&"test"));
        assert!(names.contains(&"build"));
        assert!(names.contains(&"check"));
        assert!(names.contains(&"clippy"));
        assert!(runnables
            .iter()
            .any(|r| r.name == "run" && r.command == "cargo run"));
    }

    #[test]
    fn extracts_bin_targets() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("Cargo.toml");
        fs::write(
            &path,
            r#"
[package]
name = "example"
version = "0.1.0"

[[bin]]
name = "cli"

[[bin]]
name = "daemon"
"#,
        )
        .unwrap();

        let runnables = CargoToml.extract(&path).unwrap();
        let names: Vec<_> = runnables.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"run:cli"));
        assert!(names.contains(&"run:daemon"));
        assert!(runnables
            .iter()
            .any(|r| r.name == "run:cli" && r.command == "cargo run --bin cli"));
    }
}
