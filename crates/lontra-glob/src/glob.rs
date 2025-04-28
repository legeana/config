use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use glob::Paths;
use glob::glob as str_glob;

pub struct Iter(Paths);

impl Iterator for Iter {
    type Item = Result<PathBuf>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next()? {
            Ok(path) => Some(Ok(path)),
            Err(e) => Some(Err(e.into())),
        }
    }
}

/// Returns an iterator that produces all the `Path`s that match the given
/// pattern relative to `root`.
pub fn glob_iter(root: impl AsRef<Path>, pattern: impl AsRef<str>) -> Result<Iter> {
    let root = root.as_ref();
    let pattern = pattern.as_ref();
    let full = root.join(pattern);
    let full_pattern = full
        .to_str()
        .ok_or_else(|| anyhow!("failed to convert {full:?} to utf-8"))?;
    log::debug!("glob({root:?}, {pattern:?}) => glob({full_pattern:?})");
    let paths = str_glob(full_pattern)?;
    Ok(Iter(paths))
}
