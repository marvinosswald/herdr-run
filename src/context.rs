use anyhow::{anyhow, Result};
use serde_json::Value;
use std::env;
use std::path::{Path, PathBuf};

pub fn herdr_bin() -> String {
    env::var("HERDR_BIN_PATH").unwrap_or_else(|_| "herdr".to_string())
}

pub fn action_id() -> Option<String> {
    env::var("HERDR_PLUGIN_ACTION_ID").ok()
}

pub fn entrypoint_id() -> Option<String> {
    env::var("HERDR_PLUGIN_ENTRYPOINT_ID").ok()
}

pub fn context_json() -> Result<Value> {
    let raw = env::var("HERDR_PLUGIN_CONTEXT_JSON").unwrap_or_else(|_| "{}".to_string());
    serde_json::from_str(&raw).map_err(|e| anyhow!("failed to parse context JSON: {e}"))
}

pub fn project_cwd() -> Result<PathBuf> {
    let ctx = context_json()?;

    if let Some(cwd) = ctx.get("focused_pane_cwd").and_then(|v| v.as_str()) {
        return Ok(PathBuf::from(cwd));
    }

    if let Some(cwd) = ctx.get("workspace_cwd").and_then(|v| v.as_str()) {
        return Ok(PathBuf::from(cwd));
    }

    if let Some(cwd) = ctx
        .get("workspace")
        .and_then(|w| w.get("cwd"))
        .and_then(|v| v.as_str())
    {
        return Ok(PathBuf::from(cwd));
    }

    env::current_dir().map_err(|e| anyhow!("no cwd in context and cannot read current dir: {e}"))
}

pub fn git_root(start: &Path) -> Option<PathBuf> {
    let output = std::process::Command::new("git")
        .args([
            "-C",
            start.to_str().unwrap_or("."),
            "rev-parse",
            "--show-toplevel",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let root = String::from_utf8(output.stdout).ok()?;
    Some(PathBuf::from(root.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::TempDir;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    const VARS: &[&str] = &[
        "HERDR_BIN_PATH",
        "HERDR_PLUGIN_ACTION_ID",
        "HERDR_PLUGIN_ENTRYPOINT_ID",
        "HERDR_PLUGIN_CONTEXT_JSON",
    ];

    fn with_clean_env<F: FnOnce()>(f: F) {
        let _guard = ENV_LOCK.lock().unwrap();
        for v in VARS {
            env::remove_var(v);
        }
        f();
        for v in VARS {
            env::remove_var(v);
        }
    }

    #[test]
    fn herdr_bin_defaults_to_herdr() {
        with_clean_env(|| assert_eq!(herdr_bin(), "herdr"));
    }

    #[test]
    fn herdr_bin_uses_env_override() {
        with_clean_env(|| {
            env::set_var("HERDR_BIN_PATH", "/custom/herdr");
            assert_eq!(herdr_bin(), "/custom/herdr");
        });
    }

    #[test]
    fn ids_are_none_when_unset() {
        with_clean_env(|| {
            assert!(action_id().is_none());
            assert!(entrypoint_id().is_none());
        });
    }

    #[test]
    fn ids_read_from_env() {
        with_clean_env(|| {
            env::set_var("HERDR_PLUGIN_ACTION_ID", "a1");
            env::set_var("HERDR_PLUGIN_ENTRYPOINT_ID", "e1");
            assert_eq!(action_id().as_deref(), Some("a1"));
            assert_eq!(entrypoint_id().as_deref(), Some("e1"));
        });
    }

    #[test]
    fn context_json_defaults_to_empty_object() {
        with_clean_env(|| assert_eq!(context_json().unwrap(), Value::Object(Default::default())));
    }

    #[test]
    fn context_json_parses_env() {
        with_clean_env(|| {
            env::set_var("HERDR_PLUGIN_CONTEXT_JSON", r#"{"foo":1}"#);
            assert_eq!(context_json().unwrap()["foo"], 1);
        });
    }

    #[test]
    fn context_json_errors_on_invalid() {
        with_clean_env(|| {
            env::set_var("HERDR_PLUGIN_CONTEXT_JSON", "not json");
            assert!(context_json().is_err());
        });
    }

    #[test]
    fn project_cwd_prefers_focused_pane() {
        with_clean_env(|| {
            env::set_var(
                "HERDR_PLUGIN_CONTEXT_JSON",
                r#"{"focused_pane_cwd":"/focused","workspace_cwd":"/workspace"}"#,
            );
            assert_eq!(project_cwd().unwrap(), PathBuf::from("/focused"));
        });
    }

    #[test]
    fn project_cwd_falls_back_to_workspace_cwd() {
        with_clean_env(|| {
            env::set_var(
                "HERDR_PLUGIN_CONTEXT_JSON",
                r#"{"workspace_cwd":"/workspace","workspace":{"cwd":"/nested"}}"#,
            );
            assert_eq!(project_cwd().unwrap(), PathBuf::from("/workspace"));
        });
    }

    #[test]
    fn project_cwd_falls_back_to_workspace_object() {
        with_clean_env(|| {
            env::set_var("HERDR_PLUGIN_CONTEXT_JSON", r#"{"workspace":{"cwd":"/nested"}}"#);
            assert_eq!(project_cwd().unwrap(), PathBuf::from("/nested"));
        });
    }

    #[test]
    fn project_cwd_falls_back_to_current_dir() {
        with_clean_env(|| {
            env::set_var("HERDR_PLUGIN_CONTEXT_JSON", "{}");
            assert_eq!(project_cwd().unwrap(), env::current_dir().unwrap());
        });
    }

    #[test]
    fn git_root_returns_root_in_repo() {
        let dir = TempDir::new().unwrap();
        let output = std::process::Command::new("git")
            .args(["init", dir.path().to_str().unwrap()])
            .output()
            .expect("git init failed");
        assert!(output.status.success());

        let sub = dir.path().join("sub");
        std::fs::create_dir(&sub).unwrap();
        assert_eq!(
            git_root(&sub).unwrap(),
            dir.path().canonicalize().unwrap()
        );
    }

    #[test]
    fn git_root_returns_none_outside_repo() {
        let dir = TempDir::new().unwrap();
        assert!(git_root(dir.path()).is_none());
    }
}
