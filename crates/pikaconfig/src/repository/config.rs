use std::path::Path;

use anyhow::{Context as _, Result};
use serde::Deserialize;

use crate::tag_criteria;

const REPOSITORY_CONFIG_TOML: &str = "repository.toml";

/// repository.toml file definition
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct Repository {
    pub requires: Option<tag_criteria::TagCriteria>,
}

fn load_toml_string(data: &str) -> Result<Repository> {
    toml::from_str(data).context("failed to deserialize Repository")
}

fn load_toml_file(config_path: &Path) -> Result<Repository> {
    let input = std::fs::read_to_string(config_path)
        .with_context(|| format!("failed to read {config_path:?}"))?;
    load_toml_string(&input).with_context(|| format!("failed to parse {config_path:?}"))
}

pub(super) fn load_repository(root: &Path) -> Result<Repository> {
    load_toml_file(&root.join(REPOSITORY_CONFIG_TOML))
}

pub fn is_repository_dir(root: &Path) -> Result<bool> {
    Ok(root.join(REPOSITORY_CONFIG_TOML).try_exists()?)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::string_list::StringList;

    use super::*;

    #[test]
    fn test_load_empty_string() {
        let repo = load_toml_string("").expect("load_toml_string");
        assert_eq!(repo.requires, None);
    }

    #[test]
    fn test_load_header() {
        let repo = load_toml_string(
            "
            requires = ['r1', 'r2']
            ",
        )
        .expect("load_toml_string");

        assert_eq!(
            repo.requires,
            Some(tag_criteria::TagCriteria(StringList::List(vec![
                "r1".to_owned(),
                "r2".to_owned(),
            ]))),
        );
    }
}
