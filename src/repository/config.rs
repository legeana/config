use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

use crate::file_util;
use crate::tag_criteria;

const REPOSITORY_CONFIG_TOML: &str = "repository.toml";
const REPOSITORY_CONFIG_YAML: &str = "repository.yaml";

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

fn load_toml_string(data: &str) -> Result<Repository> {
    toml::from_str(data).context("failed to deserialize Repository")
}

fn load_yaml_string(data: &str) -> Result<Repository> {
    serde_yaml::from_str(data).context("failed to deserialize Repository")
}

fn load_toml_file(config_path: &Path) -> Result<Repository> {
    let input = std::fs::read_to_string(config_path)
        .with_context(|| format!("failed to read {config_path:?}"))?;
    load_toml_string(&input).with_context(|| format!("failed to parse {config_path:?}"))
}

fn load_yaml_file(config_path: &Path) -> Result<Repository> {
    let input = std::fs::read_to_string(config_path)
        .with_context(|| format!("failed to read {config_path:?}"))?;
    load_yaml_string(&input).with_context(|| format!("failed to parse {config_path:?}"))
}

pub fn load_repository(root: &Path) -> Result<Repository> {
    let mut repos: Vec<Repository> = vec![
        file_util::if_found(load_toml_file(&root.join(REPOSITORY_CONFIG_TOML)))?,
        file_util::if_found(load_yaml_file(&root.join(REPOSITORY_CONFIG_YAML)))?,
    ]
    .into_iter()
    .filter_map(|repo| repo)
    .collect();
    match repos.len() {
        0 => Err(anyhow!("{root:?} is not a repository")),
        1 => Ok(repos.pop().expect("must not be empty")),
        _ => Err(anyhow!("{root:?} has multiple repository.* files")),
    }
}

pub fn is_repository_dir(root: &Path) -> Result<bool> {
    let toml = root.join(REPOSITORY_CONFIG_TOML).try_exists()?;
    let yaml = root.join(REPOSITORY_CONFIG_YAML).try_exists()?;
    Ok(toml || yaml)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_empty_string() {
        let repo = load_toml_string("").expect("load_toml_string");
        assert_eq!(repo.requires, None);
        assert_eq!(repo.conflicts, None);
    }

    #[test]
    fn test_load_example() {
        let repo = load_toml_string(
            "
            requires = ['r1', 'r2']
            conflicts = ['c1', 'c2']
            ",
        )
        .expect("load_toml_string");

        assert_eq!(repo.requires, Some(vec!["r1".to_owned(), "r2".to_owned()]));
        assert_eq!(repo.conflicts, Some(vec!["c1".to_owned(), "c2".to_owned()]));
    }
}
