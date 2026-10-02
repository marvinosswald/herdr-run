use crate::{herdr_cli, picker, runnable::Runnable, state};
use anyhow::{Context, Result};
use std::env;
use std::path::PathBuf;

pub fn picker() -> Result<()> {
    let runnables_json = env::var("RUNNABLES_JSON").context("RUNNABLES_JSON not set")?;
    let project_root = env::var("PROJECT_ROOT").context("PROJECT_ROOT not set")?;
    let mut runnables: Vec<Runnable> = serde_json::from_str(&runnables_json)?;
    let project_root = PathBuf::from(project_root);

    if let Some(default) = state::get_default(&project_root) {
        for r in &mut runnables {
            if r.source_file == default.source_file && r.name == default.name {
                r.is_default = true;
                break;
            }
        }
    }

    if let Some((selected, set_default)) = picker::select(&runnables)? {
        if set_default {
            state::set_default(&project_root, &selected)?;
        }
        herdr_cli::run_in_new_tab(&project_root, &selected)?;
    }

    Ok(())
}
