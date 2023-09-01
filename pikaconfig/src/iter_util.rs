use std::iter::IntoIterator;

use anyhow::{anyhow, Result};

/// Try to return a single element from a collection.
/// Ok(element) if collection contains a single element.
/// Ok(None) if collection is empty.
/// Err(...) if there are multiple elements.
pub fn to_option<T, Item>(collection: T) -> Result<Option<Item>>
where
    T: IntoIterator<Item = Item>,
{
    collection
        .into_iter()
        .try_fold(None, |acc, value| match acc {
            None => Ok(Some(value)),
            Some(_) => Err(anyhow!("too many elements")),
        })
}
