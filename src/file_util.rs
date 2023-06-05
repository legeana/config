use std::fs;
use std::io;
use std::path::Path;

use anyhow::Result;

/// Same as std::fs::read_to_string() but returns None if the file doesn't exist.
pub fn try_read_to_string(path: &Path) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(data) => Ok(Some(data)),
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => Ok(None),
            _ => Err(err.into()),
        },
    }
}
