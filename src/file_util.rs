use std::io;

use anyhow::{Error, Result};

pub fn is_not_found(err: &Error) -> bool {
    match err.downcast_ref::<io::Error>() {
        Some(err) => err.kind() == io::ErrorKind::NotFound,
        None => false,
    }
}

/// Returns Ok(None) on std::io::ErrorKind::NotFound, result otherwise.
pub fn skip_not_found<T>(result: Result<T>) -> Result<Option<T>> {
    match result {
        Ok(t) => Ok(Some(t)),
        Err(err) => {
            if is_not_found(&err) {
                Ok(None)
            } else {
                Err(err)
            }
        }
    }
}
