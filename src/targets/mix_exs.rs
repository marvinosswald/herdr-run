use super::{runnable, RunnableTarget};
use crate::runnable::Runnable;
use anyhow::Result;
use std::path::Path;

pub struct MixExs;

impl RunnableTarget for MixExs {
    fn kind(&self) -> &'static str {
        "mix.exs"
    }

    fn file_names(&self) -> &'static [&'static str] {
        &["mix.exs"]
    }

    fn extract(&self, path: &Path) -> Result<Vec<Runnable>> {
        let kind = self.kind();
        Ok(vec![
            runnable(path, kind, "test", "mix test"),
            runnable(path, kind, "compile", "mix compile"),
            runnable(path, kind, "run", "mix run"),
            runnable(path, kind, "deps.get", "mix deps.get"),
            runnable(path, kind, "format.check", "mix format --check-formatted"),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn extracts_fixed_mix_runnables() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("mix.exs");
        fs::write(&path, "defmodule MixProject do\nend\n").unwrap();

        let runnables = MixExs.extract(&path).unwrap();
        let names: Vec<_> = runnables.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"test"));
        assert!(names.contains(&"compile"));
        assert!(names.contains(&"run"));
        assert!(names.contains(&"deps.get"));
        assert!(names.contains(&"format.check"));
        assert!(runnables
            .iter()
            .any(|r| r.name == "format.check" && r.command == "mix format --check-formatted"));
    }
}
