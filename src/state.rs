use crate::runnable::Runnable;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn state_dir() -> Result<PathBuf> {
    env::var("HERDR_PLUGIN_STATE_DIR")
        .map(PathBuf::from)
        .context("HERDR_PLUGIN_STATE_DIR is not set")
}

fn defaults_path() -> Result<PathBuf> {
    Ok(state_dir()?.join("defaults.json"))
}

fn load_defaults() -> Result<HashMap<String, Runnable>> {
    let path = defaults_path()?;
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let raw =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&raw).context("failed to parse defaults.json")
}

fn save_defaults(defaults: &HashMap<String, Runnable>) -> Result<()> {
    let path = defaults_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let raw = serde_json::to_string_pretty(defaults)?;
    fs::write(&path, raw).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub fn get_default(project_root: &Path) -> Option<Runnable> {
    let key = project_root.to_string_lossy().to_string();
    load_defaults().ok()?.get(&key).cloned()
}

pub fn set_default(project_root: &Path, runnable: &Runnable) -> Result<()> {
    let mut defaults = load_defaults()?;
    defaults.insert(project_root.to_string_lossy().to_string(), runnable.clone());
    save_defaults(&defaults)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::TempDir;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_state_dir<F: FnOnce(&Path)>(f: F) {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = TempDir::new().unwrap();
        env::set_var("HERDR_PLUGIN_STATE_DIR", dir.path());
        f(dir.path());
        env::remove_var("HERDR_PLUGIN_STATE_DIR");
    }

    fn sample_runnable(name: &str) -> Runnable {
        Runnable {
            target_kind: "package.json".to_string(),
            source_file: PathBuf::from("/tmp/package.json"),
            name: name.to_string(),
            command: format!("npm run {name}"),
            is_default: false,
        }
    }

    #[test]
    fn get_default_returns_none_without_state_file() {
        with_state_dir(|_| {
            assert!(get_default(Path::new("/project")).is_none());
        });
    }

    #[test]
    fn set_and_get_default_round_trip() {
        with_state_dir(|dir| {
            let root = Path::new("/project");
            let r = sample_runnable("build");
            set_default(root, &r).unwrap();

            assert!(dir.join("defaults.json").exists());
            let loaded = get_default(root).unwrap();
            assert_eq!(loaded.name, "build");
            assert_eq!(loaded.command, "npm run build");
        });
    }

    #[test]
    fn get_default_returns_none_for_unknown_root() {
        with_state_dir(|_| {
            set_default(Path::new("/project"), &sample_runnable("build")).unwrap();
            assert!(get_default(Path::new("/other")).is_none());
        });
    }

    #[test]
    fn set_default_overwrites_and_preserves_other_keys() {
        with_state_dir(|_| {
            set_default(Path::new("/a"), &sample_runnable("one")).unwrap();
            set_default(Path::new("/b"), &sample_runnable("two")).unwrap();
            set_default(Path::new("/a"), &sample_runnable("three")).unwrap();

            assert_eq!(get_default(Path::new("/a")).unwrap().name, "three");
            assert_eq!(get_default(Path::new("/b")).unwrap().name, "two");
        });
    }
}
