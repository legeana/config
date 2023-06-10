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
    let iter = collection.into_iter();
    iter.fold(Ok(None), |acc, value| match acc {
        Ok(None) => Ok(Some(value)),
        Ok(Some(_)) => Err(anyhow!("too many elements")),
        Err(err) => Err(err),
    })
}
