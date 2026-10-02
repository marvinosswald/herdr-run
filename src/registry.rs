use crate::targets::cargo_toml::CargoToml;
use crate::targets::composer_json::ComposerJson;
use crate::targets::justfile::Justfile;
use crate::targets::makefile::Makefile;
use crate::targets::mix_exs::MixExs;
use crate::targets::package_json::PackageJson;
use crate::targets::pyproject_toml::PyprojectToml;
use crate::targets::RunnableTarget;

pub fn default_targets() -> Vec<Box<dyn RunnableTarget>> {
    vec![
        Box::new(PackageJson),
        Box::new(ComposerJson),
        Box::new(Makefile),
        Box::new(Justfile),
        Box::new(CargoToml),
        Box::new(PyprojectToml),
        Box::new(MixExs),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn default_targets_covers_all_kinds() {
        let targets = default_targets();
        let kinds: HashSet<_> = targets.iter().map(|t| t.kind()).collect();
        for expected in [
            "package.json",
            "composer.json",
            "Makefile",
            "justfile",
            "Cargo.toml",
            "pyproject.toml",
            "mix.exs",
        ] {
            assert!(kinds.contains(expected), "missing target {expected}");
        }
        assert_eq!(targets.len(), 7);
    }

    #[test]
    fn file_names_have_no_duplicates() {
        let mut seen = HashSet::new();
        for target in default_targets() {
            for name in target.file_names() {
                assert!(seen.insert(*name), "duplicate file name {name}");
            }
        }
    }
}
