use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Runnable {
    pub target_kind: String,
    pub source_file: PathBuf,
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub is_default: bool,
}

impl Runnable {
    pub fn display_label(&self) -> String {
        let marker = if self.is_default { "* " } else { "" };
        format!(
            "{}{}: {} — {}",
            marker, self.target_kind, self.name, self.command
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_runnable() -> Runnable {
        Runnable {
            target_kind: "package.json".to_string(),
            source_file: PathBuf::from("/tmp/package.json"),
            name: "build".to_string(),
            command: "npm run build".to_string(),
            is_default: false,
        }
    }

    #[test]
    fn display_label_without_default_marker() {
        let r = sample_runnable();
        assert_eq!(r.display_label(), "package.json: build — npm run build");
    }

    #[test]
    fn display_label_with_default_marker() {
        let mut r = sample_runnable();
        r.is_default = true;
        assert_eq!(r.display_label(), "* package.json: build — npm run build");
    }
}
