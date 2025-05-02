use std::fs;
use std::io;
use std::path::Path;

use anyhow::{Context as _, Result, anyhow};

pub(crate) struct Metadata {
    metadata: fs::Metadata,
}

impl Metadata {
    #[allow(dead_code)]
    pub(crate) fn is_symlink(&self) -> bool {
        self.metadata.is_symlink()
    }
    #[cfg(unix)]
    pub(crate) fn is_symlink_file(&self) -> bool {
        self.metadata.is_symlink()
    }
    #[cfg(windows)]
    pub(crate) fn is_symlink_file(&self) -> bool {
        use std::os::windows::fs::FileTypeExt as _;
        self.metadata.file_type().is_symlink_file()
    }
    #[cfg(unix)]
    pub(crate) fn is_symlink_dir(&self) -> bool {
        false
    }
    #[cfg(windows)]
    pub(crate) fn is_symlink_dir(&self) -> bool {
        use std::os::windows::fs::FileTypeExt as _;
        self.metadata.file_type().is_symlink_dir()
    }
}

impl From<Metadata> for fs::Metadata {
    fn from(symlink_metadata: Metadata) -> Self {
        symlink_metadata.metadata
    }
}

impl From<fs::Metadata> for Metadata {
    fn from(metadata: fs::Metadata) -> Self {
        Self { metadata }
    }
}

pub(crate) fn metadata(path: &Path) -> io::Result<Metadata> {
    let metadata = path.symlink_metadata()?;
    Ok(metadata.into())
}

/// Remove path if it is a symlink.
pub fn remove(path: &Path) -> Result<()> {
    let md: Metadata =
        metadata(path).with_context(|| format!("failed to get {path:?} metadata"))?;
    if md.is_symlink_file() {
        fs::remove_file(path)?;
        Ok(())
    } else if md.is_symlink_dir() {
        fs::remove_dir(path)?;
        Ok(())
    } else {
        Err(anyhow!("{path:?} is not a symlink"))
    }
}
