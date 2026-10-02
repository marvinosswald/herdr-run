use crate::context;
use crate::runnable::Runnable;
use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::path::Path;
use std::process::Command;

pub fn run_in_new_tab(cwd: &Path, runnable: &Runnable) -> Result<()> {
    let herdr = context::herdr_bin();
    let label = format!("{}: {}", runnable.target_kind, runnable.name);

    let output = Command::new(&herdr)
        .args([
            "tab",
            "create",
            "--cwd",
            cwd.to_str().unwrap_or("."),
            "--label",
            &label,
            "--focus",
        ])
        .output()
        .with_context(|| format!("failed to spawn `{herdr} tab create`"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("`{herdr} tab create` failed: {stderr}");
    }

    let json: Value = serde_json::from_slice(&output.stdout)
        .context("failed to parse `herdr tab create` JSON response")?;

    let pane_id = json
        .get("result")
        .and_then(|r| r.get("root_pane"))
        .and_then(|p| p.get("pane_id"))
        .and_then(|v| v.as_str())
        .context("`herdr tab create` response missing root_pane.pane_id")?;

    let run_output = Command::new(&herdr)
        .args(["pane", "run", pane_id, &runnable.command])
        .output()
        .with_context(|| format!("failed to spawn `{herdr} pane run`"))?;

    if !run_output.status.success() {
        let stderr = String::from_utf8_lossy(&run_output.stderr);
        bail!("`{herdr} pane run` failed: {stderr}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use tempfile::TempDir;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn sample_runnable() -> Runnable {
        Runnable {
            target_kind: "package.json".to_string(),
            source_file: PathBuf::from("/tmp/package.json"),
            name: "build".to_string(),
            command: "npm run build".to_string(),
            is_default: false,
        }
    }

    /// Writes a fake `herdr` executable that logs its args to $LOG_FILE.
    /// `tab_create_stdout` is printed when invoked as `tab create`.
    fn fake_herdr(dir: &Path, tab_create_stdout: &str, exit_code: i32) -> PathBuf {
        let path = dir.join("herdr");
        let script = format!(
            r#"#!/bin/sh
echo "$@" >> "$LOG_FILE"
if [ "$1" = "tab" ]; then
  cat <<'EOF'
{tab_create_stdout}
EOF
fi
exit {exit_code}
"#
        );
        fs::write(&path, script).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        }
        path
    }

    #[test]
    fn run_in_new_tab_succeeds_and_runs_command() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = TempDir::new().unwrap();
        let log = dir.path().join("calls.log");
        let bin = fake_herdr(
            dir.path(),
            r#"{"result":{"root_pane":{"pane_id":"p1"}}}"#,
            0,
        );
        std::env::set_var("HERDR_BIN_PATH", &bin);
        std::env::set_var("LOG_FILE", &log);

        run_in_new_tab(Path::new("/project"), &sample_runnable()).unwrap();

        let calls = fs::read_to_string(&log).unwrap();
        assert!(calls.contains("tab create --cwd /project --label package.json: build --focus"));
        assert!(calls.contains("pane run p1 npm run build"));

        std::env::remove_var("HERDR_BIN_PATH");
        std::env::remove_var("LOG_FILE");
    }

    #[test]
    fn run_in_new_tab_errors_when_tab_create_fails() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = TempDir::new().unwrap();
        let log = dir.path().join("calls.log");
        let bin = fake_herdr(dir.path(), "{}", 1);
        std::env::set_var("HERDR_BIN_PATH", &bin);
        std::env::set_var("LOG_FILE", &log);

        assert!(run_in_new_tab(Path::new("/project"), &sample_runnable()).is_err());

        std::env::remove_var("HERDR_BIN_PATH");
        std::env::remove_var("LOG_FILE");
    }

    #[test]
    fn run_in_new_tab_errors_on_malformed_json() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = TempDir::new().unwrap();
        let log = dir.path().join("calls.log");
        let bin = fake_herdr(dir.path(), "not json", 0);
        std::env::set_var("HERDR_BIN_PATH", &bin);
        std::env::set_var("LOG_FILE", &log);

        assert!(run_in_new_tab(Path::new("/project"), &sample_runnable()).is_err());

        std::env::remove_var("HERDR_BIN_PATH");
        std::env::remove_var("LOG_FILE");
    }

    #[test]
    fn run_in_new_tab_errors_when_pane_id_missing() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = TempDir::new().unwrap();
        let log = dir.path().join("calls.log");
        let bin = fake_herdr(dir.path(), r#"{"result":{}}"#, 0);
        std::env::set_var("HERDR_BIN_PATH", &bin);
        std::env::set_var("LOG_FILE", &log);

        assert!(run_in_new_tab(Path::new("/project"), &sample_runnable()).is_err());

        std::env::remove_var("HERDR_BIN_PATH");
        std::env::remove_var("LOG_FILE");
    }
}
