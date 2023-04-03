use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

const REPOSITORY_CONFIG_NAME: &str = "repository.toml";

/// repository.toml file definition
#[derive(Deserialize, PartialEq, Eq, Default, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Repository {
    /// Required tags.
    pub requires: Option<Vec<String>>,
    /// Conflicting tags.
    pub conflicts: Option<Vec<String>>,
}

pub fn load_string(data: &str) -> Result<Repository> {
    let deserializer = toml::Deserializer::new(data);
    let pkg = Repository::deserialize(deserializer).context("failed to deserialize Repository")?;
    Ok(pkg)
}

pub fn load_file(config_path: &Path) -> Result<Repository> {
    let raw_input =
        std::fs::read(config_path).with_context(|| format!("failed to read {config_path:?}"))?;
    let input = String::from_utf8(raw_input)
        .with_context(|| format!("failed to convert {config_path:?} to utf8"))?;
    let pkg = load_string(&input).with_context(|| format!("failed to load {config_path:?}"))?;
    Ok(pkg)
}

pub fn load_repository(root: &Path) -> Result<Repository> {
    load_file(&root.join(REPOSITORY_CONFIG_NAME))
}

pub fn is_repository_dir(root: &Path) -> Result<bool> {
    Ok(root.join(REPOSITORY_CONFIG_NAME).try_exists()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_empty_string() {
        let repo = load_string("").expect("load_string");
        assert_eq!(repo.requires, None);
        assert_eq!(repo.conflicts, None);
    }

    #[test]
    fn test_load_example() {
        let repo = load_string(
            "
            requires = ['r1', 'r2']
            conflicts = ['c1', 'c2']
            ",
        )
        .expect("load_string");

        assert_eq!(repo.requires, Some(vec!["r1".to_owned(), "r2".to_owned()]));
        assert_eq!(repo.conflicts, Some(vec!["c1".to_owned(), "c2".to_owned()]));
    }
}
