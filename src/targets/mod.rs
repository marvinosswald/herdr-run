use crate::runnable::Runnable;
use anyhow::Result;
use std::path::Path;

pub mod cargo_toml;
pub mod composer_json;
pub mod justfile;
pub mod makefile;
pub mod mix_exs;
pub mod package_json;
pub mod pyproject_toml;

pub trait RunnableTarget: Send + Sync {
    fn kind(&self) -> &'static str;
    fn file_names(&self) -> &'static [&'static str];
    fn extract(&self, path: &Path) -> Result<Vec<Runnable>>;
}

pub fn runnable(
    path: &Path,
    kind: &str,
    name: impl Into<String>,
    command: impl Into<String>,
) -> Runnable {
    Runnable {
        target_kind: kind.to_string(),
        source_file: path.to_path_buf(),
        name: name.into(),
        command: command.into(),
        is_default: false,
    }
}
