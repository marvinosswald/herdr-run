use crate::runnable::Runnable;
use anyhow::{bail, Context, Result};
use std::io::{self, BufRead, Write};
use std::process::{Command, Stdio};

pub fn select(runnables: &[Runnable]) -> Result<Option<(Runnable, bool)>> {
    if runnables.is_empty() {
        return Ok(None);
    }

    if fzf_available() {
        fzf_select(runnables)
    } else {
        fallback_select(runnables)
    }
}

fn fzf_available() -> bool {
    Command::new("fzf")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn fzf_select(runnables: &[Runnable]) -> Result<Option<(Runnable, bool)>> {
    let input = runnables
        .iter()
        .map(|r| {
            let display = r.display_label();
            let json = serde_json::to_string(r).unwrap();
            format!("{display}\t{json}")
        })
        .collect::<Vec<_>>()
        .join("\n");

    let mut child = Command::new("fzf")
        .args([
            "--delimiter=\t",
            "--with-nth=1",
            "--expect=ctrl-d",
            "--header=Enter: run  |  Ctrl-D: set as default and run",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("failed to spawn fzf")?;

    {
        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all(input.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    let code = output.status.code().unwrap_or(130);

    if code == 130 {
        return Ok(None);
    }
    if code != 0 {
        bail!("fzf exited with status {code}");
    }

    let text = String::from_utf8(output.stdout)?;
    parse_fzf_output(&text).map(Some)
}

fn parse_fzf_output(text: &str) -> Result<(Runnable, bool)> {
    let mut lines = text.lines();
    let key = lines.next().unwrap_or("");
    let selection = lines.next().context("fzf produced no selection")?;

    let json = selection
        .split_once('\t')
        .map(|x| x.1)
        .context("malformed fzf output")?;

    let runnable: Runnable = serde_json::from_str(json)?;
    Ok((runnable, key == "ctrl-d"))
}

fn fallback_select(runnables: &[Runnable]) -> Result<Option<(Runnable, bool)>> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();

    println!("Available runnables:");
    for (i, r) in runnables.iter().enumerate() {
        println!("  {}. {}", i + 1, r.display_label());
    }
    print!("Select number: ");
    io::stdout().flush()?;

    let mut line = String::new();
    reader.read_line(&mut line)?;

    let n: usize = line.trim().parse().context("invalid selection")?;

    let runnable = runnables
        .get(n.saturating_sub(1))
        .context("selection out of range")?
        .clone();

    print!("Set as default? [y/N] ");
    io::stdout().flush()?;
    line.clear();
    reader.read_line(&mut line)?;

    let set_default = line.trim().eq_ignore_ascii_case("y");
    Ok(Some((runnable, set_default)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_runnable() -> Runnable {
        Runnable {
            target_kind: "justfile".to_string(),
            source_file: PathBuf::from("/tmp/justfile"),
            name: "build".to_string(),
            command: "just build".to_string(),
            is_default: false,
        }
    }

    #[test]
    fn select_returns_none_for_empty_list() {
        assert!(select(&[]).unwrap().is_none());
    }

    #[test]
    fn parse_fzf_output_with_enter_key() {
        let json = serde_json::to_string(&sample_runnable()).unwrap();
        let text = format!("\njustfile: build — just build\t{json}\n");

        let (r, set_default) = parse_fzf_output(&text).unwrap();
        assert_eq!(r.name, "build");
        assert_eq!(r.command, "just build");
        assert!(!set_default);
    }

    #[test]
    fn parse_fzf_output_with_ctrl_d() {
        let json = serde_json::to_string(&sample_runnable()).unwrap();
        let text = format!("ctrl-d\nlabel\t{json}\n");

        let (_, set_default) = parse_fzf_output(&text).unwrap();
        assert!(set_default);
    }

    #[test]
    fn parse_fzf_output_errors_without_selection() {
        assert!(parse_fzf_output("ctrl-d\n").is_err());
    }

    #[test]
    fn parse_fzf_output_errors_on_missing_tab() {
        assert!(parse_fzf_output("\nno tab here\n").is_err());
    }
}
