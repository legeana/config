use anyhow::{Context, Result};

use crate::tag_util;

pub trait TagCriteria {
    fn requires(&self) -> Option<&[String]>;
    fn conflicts(&self) -> Option<&[String]>;
}

pub fn is_satisfied<T: TagCriteria>(criteria: &T) -> Result<bool> {
    if let Some(requires) = criteria.requires() {
        if !tag_util::has_all_tags(requires)
            .with_context(|| format!("failed to check tags {requires:?}"))?
        {
            return Ok(false);
        }
    }
    if let Some(conflicts) = criteria.conflicts() {
        if tag_util::has_any_tags(conflicts)
            .with_context(|| format!("failed to check tags {conflicts:?}"))?
        {
            return Ok(false);
        }
    }
    return Ok(true);
}

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::Result;

    struct Tags {
        requires: Option<Vec<String>>,
        conflicts: Option<Vec<String>>,
    }

    impl TagCriteria for Tags {
        fn requires(&self) -> Option<&[String]> {
            self.requires.as_ref().map(|v| v.as_slice())
        }
        fn conflicts(&self) -> Option<&[String]> {
            self.conflicts.as_ref().map(|v| v.as_slice())
        }
    }

    // TODO: Make more generic tests.
    #[cfg(target_os = "linux")]
    #[test]
    fn test_satisfied() -> Result<()> {
        let tags = Tags {
            requires: Some(vec!["os=linux".to_owned()]),
            conflicts: Some(vec!["os=windows".to_owned()]),
        };
        assert!(is_satisfied(&tags)?);
        Ok(())
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_not_satisfied_requires() -> Result<()> {
        let tags = Tags {
            requires: Some(vec!["os=windows".to_owned()]),
            conflicts: None,
        };
        assert!(!is_satisfied(&tags)?);
        Ok(())
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_not_satisfied_conflicts() -> Result<()> {
        let tags = Tags {
            requires: None,
            conflicts: Some(vec!["os=linux".to_owned()]),
        };
        assert!(!is_satisfied(&tags)?);
        Ok(())
    }
}
