mod actions;
mod context;
mod herdr_cli;
mod panes;
mod picker;
mod registry;
mod runnable;
mod scanner;
mod state;
mod targets;

use crate::context::{action_id, entrypoint_id};
use anyhow::{bail, Result};

fn main() -> Result<()> {
    match (action_id(), entrypoint_id()) {
        (Some(id), _) => match id.as_str() {
            "pick" => actions::pick(),
            "run-default" => actions::run_default(),
            other => bail!("unknown action id: {other}"),
        },
        (_, Some(id)) => match id.as_str() {
            "picker" => panes::picker(),
            other => bail!("unknown pane entrypoint id: {other}"),
        },
        (None, None) => bail!("expected HERDR_PLUGIN_ACTION_ID or HERDR_PLUGIN_ENTRYPOINT_ID"),
    }
}
