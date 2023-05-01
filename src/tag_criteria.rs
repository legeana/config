use anyhow::{Context, Result};

use crate::tag_util;

pub trait TagCriteria {
    fn requires(&self) -> Option<&[String]>;
    fn conflicts(&self) -> Option<&[String]>;
    fn is_satisfied(&self) -> Result<bool> {
        if let Some(requires) = self.requires() {
            if !tag_util::has_all_tags(requires)
                .with_context(|| format!("failed to check tags {requires:?}"))?
            {
                return Ok(false);
            }
        }
        if let Some(conflicts) = self.conflicts() {
            if tag_util::has_any_tags(conflicts)
                .with_context(|| format!("failed to check tags {conflicts:?}"))?
            {
                return Ok(false);
            }
        }
        return Ok(true);
    }
}

pub struct Criteria {
    pub requires: Option<Vec<String>>,
    pub conflicts: Option<Vec<String>>,
}

impl TagCriteria for Criteria {
    fn requires(&self) -> Option<&[String]> {
        self.requires.as_ref().map(|v| v.as_slice())
    }
    fn conflicts(&self) -> Option<&[String]> {
        self.conflicts.as_ref().map(|v| v.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::Result;

    // TODO: Make more generic tests.
    #[cfg(target_os = "linux")]
    #[test]
    fn test_satisfied() -> Result<()> {
        let tags = Criteria {
            requires: Some(vec!["os=linux".to_owned()]),
            conflicts: Some(vec!["os=windows".to_owned()]),
        };
        assert!(tags.is_satisfied()?);
        Ok(())
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_not_satisfied_requires() -> Result<()> {
        let tags = Criteria {
            requires: Some(vec!["os=windows".to_owned()]),
            conflicts: None,
        };
        assert!(!tags.is_satisfied()?);
        Ok(())
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_not_satisfied_conflicts() -> Result<()> {
        let tags = Criteria {
            requires: None,
            conflicts: Some(vec!["os=linux".to_owned()]),
        };
        assert!(!tags.is_satisfied()?);
        Ok(())
    }
}
