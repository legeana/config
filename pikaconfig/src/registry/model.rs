use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use super::{FilePath, FilePathBuf};

#[derive(Clone, Copy, Debug)]
pub(super) enum FilePurpose {
    User = 1,
    State = 2,
}

pub(super) fn file_type_to_sql(file: FilePath) -> (i32, &Path) {
    match file {
        FilePath::Symlink(p) => (1, p),
        FilePath::Directory(p) => (2, p),
    }
}

pub(super) fn file_type_from_sql(file_type: i32, path: PathBuf) -> Result<FilePathBuf> {
    match file_type {
        1 => Ok(FilePathBuf::Symlink(path)),
        2 => Ok(FilePathBuf::Directory(path)),
        _ => Err(anyhow!("unknown FileType {file_type}")),
    }
}

pub(super) fn path_to_sql(path: &Path) -> Vec<u8> {
    crate::os_str::to_vec(path.as_os_str().to_os_string())
}

pub(super) fn path_from_sql(path: Vec<u8>) -> Result<PathBuf> {
    Ok(crate::os_str::from_vec(path)
        .context("failed to parse path")?
        .into())
}
