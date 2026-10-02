use crate::{context, herdr_cli, registry, scanner, state};
use anyhow::{Context, Result};
use std::process::Command;

pub fn pick() -> Result<()> {
    let cwd = context::project_cwd()?;
    let targets = registry::default_targets();
    let (runnables, root) = scanner::scan(&cwd, &targets)?;

    if runnables.is_empty() {
        println!("No runnable config files found under {}", root.display());
        return Ok(());
    }

    let runnables_json = serde_json::to_string(&runnables)?;
    let herdr = context::herdr_bin();

    Command::new(&herdr)
        .args([
            "plugin",
            "pane",
            "open",
            "--plugin",
            "herdr.runnables",
            "--entrypoint",
            "picker",
            "--env",
            &format!("RUNNABLES_JSON={}", runnables_json),
            "--env",
            &format!("PROJECT_ROOT={}", root.display()),
        ])
        .output()
        .with_context(|| format!("failed to open picker pane with `{herdr}`"))?;

    Ok(())
}

pub fn run_default() -> Result<()> {
    let cwd = context::project_cwd()?;
    let targets = registry::default_targets();
    let (runnables, root) = scanner::scan(&cwd, &targets)?;

    if runnables.is_empty() {
        println!("No runnable config files found under {}", root.display());
        return Ok(());
    }

    if let Some(default) = state::get_default(&root) {
        let still_valid = runnables
            .iter()
            .any(|r| r.source_file == default.source_file && r.name == default.name);

        if still_valid {
            return herdr_cli::run_in_new_tab(&root, &default);
        }
    }

    pick()
}
