# herdr-runnables

A [Herdr](https://herdr.dev) plugin that discovers project run commands from idiomatic config files and opens them in a new tab.

## What it does

- Scans the active pane's directory up to the Git root for runnable config files.
- Press `prefix + Shift + R` to open a popup picker with all discovered commands.
- Press `prefix + R` to run the project's default command.
- Press `Ctrl + D` in the picker (or answer `y` in the fallback prompt) to mark a command as the default.

## Supported config files

| File | Extracted commands |
|------|--------------------|
| `package.json` | `npm run <script>` (detects `yarn`, `pnpm`, `bun` lockfiles) |
| `composer.json` | `composer run-script <script>` |
| `Makefile` / `makefile` / `GNUmakefile` | `make <target>` |
| `justfile` / `Justfile` / `.justfile` | `just <recipe>` |
| `Cargo.toml` | `cargo run`, `cargo test`, `cargo build`, `cargo check`, `cargo clippy`, plus `cargo run --bin <name>` |
| `pyproject.toml` | `poe <task>`, `task <task>`, `poetry run <script>` |
| `mix.exs` | `mix test`, `mix compile`, `mix run`, `mix deps.get`, `mix format --check-formatted` |

The target registry is trait-based, so adding another language/framework is a small module plus one line in the registry.

## Requirements

- [Herdr](https://herdr.dev) 0.7.0+
- [Rust](https://rustup.rs) toolchain (to build locally)
- [fzf](https://junegunn.github.io/fzf/) is strongly recommended for the popup picker. Without it, a simple numbered fallback prompt is used.

## Local development

```bash
# 1. Build the plugin binary
cargo build --release

# 2. Link it into Herdr
herdr plugin link .

# 3. Verify the actions are registered
herdr plugin action list --plugin herdr.runnables
```

## Keybindings

Add these entries to your Herdr config (e.g. `~/.config/herdr/config.toml`):

```toml
[[keys.command]]
key = "prefix+shift+r"
type = "plugin_action"
command = "herdr.runnables.pick"
description = "pick runnable"

[[keys.command]]
key = "prefix+r"
type = "plugin_action"
command = "herdr.runnables.run-default"
description = "run default runnable"
```

Reload the Herdr config or restart the server for the bindings to take effect.

## How it works

1. `pick` resolves the active pane's working directory, walks up to the Git root, and collects runnables from every supported config file it finds.
2. The collected list is passed to the `picker` popup pane via the `RUNNABLES_JSON` environment variable.
3. The picker shows the list (via `fzf` or a numbered prompt), and on selection calls `herdr tab create` followed by `herdr pane run`.
4. The default runnable is saved per project in `HERDR_PLUGIN_STATE_DIR/defaults.json`.

## Adding a new target

1. Create `src/targets/my_target.rs` and implement the `RunnableTarget` trait:

   ```rust
   use crate::runnable::Runnable;
   use crate::targets::{runnable, RunnableTarget};
   use anyhow::Result;
   use std::path::Path;

   pub struct MyTarget;

   impl RunnableTarget for MyTarget {
       fn kind(&self) -> &'static str { "my-file.toml" }
       fn file_names(&self) -> &'static [&'static str] { &["my-file.toml"] }
       fn extract(&self, path: &Path) -> Result<Vec<Runnable>> {
           Ok(vec![runnable(path, self.kind(), "example", "my-command example")])
       }
   }
   ```

2. Add it to the registry in `src/registry.rs`:

   ```rust
   use crate::targets::my_target::MyTarget;
   // ...
   Box::new(MyTarget),
   ```

3. Rebuild and, if already linked, Herdr will pick up the new binary on the next invocation.

## State

Defaults are stored under the Herdr-managed plugin state directory and keyed by the absolute Git-root path. They are not committed to the project.
