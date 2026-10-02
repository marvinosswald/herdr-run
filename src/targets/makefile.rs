use super::{runnable, RunnableTarget};
use crate::runnable::Runnable;
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub struct Makefile;

impl RunnableTarget for Makefile {
    fn kind(&self) -> &'static str {
        "Makefile"
    }

    fn file_names(&self) -> &'static [&'static str] {
        &["Makefile", "makefile", "GNUmakefile"]
    }

    fn extract(&self, path: &Path) -> Result<Vec<Runnable>> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;

        let kind = self.kind();
        let mut runnables = Vec::new();
        let mut seen = HashSet::new();

        let mut push_target = |name: String, cmd: String| {
            if seen.insert(name.clone()) {
                runnables.push(runnable(path, kind, name, cmd));
            }
        };

        for line in raw.lines() {
            if line.starts_with('\t') || line.trim().is_empty() {
                continue;
            }

            if let Some(targets) = line.strip_prefix(".PHONY:") {
                for t in targets.split_whitespace() {
                    push_target(t.to_string(), format!("make {t}"));
                }
                continue;
            }

            if line.starts_with('.') {
                continue;
            }

            if let Some((target, _)) = line.split_once(':') {
                let target = target.trim();
                if target.is_empty()
                    || target.contains('=')
                    || target.contains(|c: char| {
                        c.is_whitespace() || matches!(c, '%' | '$' | '*' | '?' | '\\' | '<' | '>')
                    })
                {
                    continue;
                }
                push_target(target.to_string(), format!("make {target}"));
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
    fn extracts_make_targets() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("Makefile");
        fs::write(
            &path,
            r#"
.PHONY: build test

build:
	echo building

test:
	echo testing

# ignored
internal-var = 1

%.o: %.c
	echo ignored
"#,
        )
        .unwrap();

        let runnables = Makefile.extract(&path).unwrap();
        let names: Vec<_> = runnables.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"build"));
        assert!(names.contains(&"test"));
        assert!(!names.contains(&"%.o"));
        assert!(runnables
            .iter()
            .any(|r| r.name == "build" && r.command == "make build"));
    }

    #[test]
    fn deduplicates_phony_and_target() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("Makefile");
        fs::write(&path, ".PHONY: build\n\nbuild:\n\techo build\n").unwrap();

        let runnables = Makefile.extract(&path).unwrap();
        let names: Vec<_> = runnables.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names.iter().filter(|&&n| n == "build").count(), 1);
    }
}
