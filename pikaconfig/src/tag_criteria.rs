use anyhow::{Context, Result};
use serde::Deserialize;

use crate::tag_util;

pub trait Criteria {
    fn is_satisfied(&self) -> Result<bool>;
}

impl<T: Criteria> Criteria for Option<T> {
    fn is_satisfied(&self) -> Result<bool> {
        match self {
            Some(c) => c.is_satisfied(),
            None => Ok(true),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields, untagged)]
pub enum TagCriteria {
    Requires(Vec<String>),
}

impl Criteria for TagCriteria {
    fn is_satisfied(&self) -> Result<bool> {
        match self {
            Self::Requires(requires) => {
                if !tag_util::has_all_tags(requires)
                    .with_context(|| format!("failed to check tags {requires:?}"))?
                {
                    return Ok(false);
                }
                Ok(true)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::Result;

    // TODO: Make more generic tests.
    #[cfg(target_os = "linux")]
    #[test]
    fn test_linux_satisfied() -> Result<()> {
        let tags = TagCriteria::Requires(vec!["os=linux".to_owned()]);
        assert!(tags.is_satisfied()?);
        Ok(())
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_linux_not_satisfied() -> Result<()> {
        let tags = TagCriteria::Requires(vec!["os=windows".to_owned()]);
        assert!(!tags.is_satisfied()?);
        Ok(())
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn test_unix_satisfied() -> Result<()> {
        let tags = TagCriteria::Requires(vec!["family=unix".to_owned()]);
        assert!(tags.is_satisfied()?);
        Ok(())
    }
}
