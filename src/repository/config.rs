use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

use crate::file_util;
use crate::tag_criteria;

const REPOSITORY_CONFIG_TOML: &str = "repository.toml";

/// repository.toml file definition
#[derive(Deserialize, PartialEq, Eq, Default, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Repository {
    /// Required tags.
    pub requires: Option<Vec<String>>,
    /// Conflicting tags.
    pub conflicts: Option<Vec<String>>,
}

impl tag_criteria::TagCriteria for Repository {
    fn requires(&self) -> Option<&[String]> {
        self.requires.as_deref()
    }
    fn conflicts(&self) -> Option<&[String]> {
        self.conflicts.as_deref()
    }
}

fn load_string_toml(data: &str) -> Result<Repository> {
    toml::from_str(data).context("failed to deserialize Repository")
}

fn try_load_file_toml(config_path: &Path) -> Result<Option<Repository>> {
    let maybe_input = file_util::try_read_to_string(config_path)
        .with_context(|| format!("failed to read {config_path:?}"))?;
    let Some(input) = maybe_input else {
        return Ok(None);
    };
    let cfg =
        load_string_toml(&input).with_context(|| format!("failed to parse {config_path:?}"))?;
    Ok(Some(cfg))
}

fn try_load_repository(root: &Path) -> Result<Option<Repository>> {
    try_load_file_toml(&root.join(REPOSITORY_CONFIG_TOML))
}

pub fn load_repository(root: &Path) -> Result<Repository> {
    let Some(cfg) = try_load_repository(root)? else {
        return Err(anyhow!("{root:?} is not a repository"));
    };
    Ok(cfg)
}

pub fn is_repository_dir(root: &Path) -> Result<bool> {
    Ok(root.join(REPOSITORY_CONFIG_TOML).try_exists()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_empty_string() {
        let repo = load_string_toml("").expect("load_string_toml");
        assert_eq!(repo.requires, None);
        assert_eq!(repo.conflicts, None);
    }

    #[test]
    fn test_load_example() {
        let repo = load_string_toml(
            "
            requires = ['r1', 'r2']
            conflicts = ['c1', 'c2']
            ",
        )
        .expect("load_string_toml");

        assert_eq!(repo.requires, Some(vec!["r1".to_owned(), "r2".to_owned()]));
        assert_eq!(repo.conflicts, Some(vec!["c1".to_owned(), "c2".to_owned()]));
    }
}
