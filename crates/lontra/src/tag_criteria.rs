use anyhow::{Context as _, Result};
use serde::Deserialize;

use crate::string_list::StringList;

pub(crate) trait Criteria {
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
#[serde(deny_unknown_fields)]
pub(crate) struct TagCriteria(pub StringList);

impl Criteria for TagCriteria {
    fn is_satisfied(&self) -> Result<bool> {
        let requires = self.0.as_slice();
        if !lontra_tags::has_all_tags(requires)
            .with_context(|| format!("failed to check tags {requires:?}"))?
        {
            return Ok(false);
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::Result;

    #[test]
    fn test_satisfied() -> Result<()> {
        let os_tag = format!("os={}", std::env::consts::OS);
        let tags = TagCriteria(StringList::List(vec![os_tag]));
        assert!(tags.is_satisfied()?);
        Ok(())
    }

    #[test]
    fn test_not_satisfied() -> Result<()> {
        let os_tag = format!("os=not-{}", std::env::consts::OS);
        let tags = TagCriteria(StringList::List(vec![os_tag]));
        assert!(!tags.is_satisfied()?);
        Ok(())
    }
}
