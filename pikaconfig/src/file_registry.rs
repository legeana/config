use crate::registry::{FilePathBuf, ImmutableRegistry};

use std::io::ErrorKind;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};

struct FileList(PathBuf);

impl FileList {
    fn list(&self) -> Result<Vec<PathBuf>> {
        if !self.0.exists() {
            return Ok(Vec::new());
        }
        if !self.0.is_file() {
            return Err(anyhow!("{:?} registry must be a file", self.0));
        }
        let data = std::fs::read_to_string(&self.0)
            .with_context(|| format!("failed to read {:?}", self.0))?;
        Ok(data
            .split('\n')
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .collect())
    }
    fn clear(&mut self) -> Result<()> {
        match std::fs::remove_file(&self.0) {
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
            r => r.with_context(|| format!("failed to remove {:?}", self.0)),
        }
    }
}

pub struct FileRegistry {
    user_files: FileList,
    state_files: FileList,
}

impl FileRegistry {
    pub fn new(user_files_path: PathBuf, state_files_path: PathBuf) -> Self {
        Self {
            user_files: FileList(user_files_path),
            state_files: FileList(state_files_path),
        }
    }
}

impl ImmutableRegistry for FileRegistry {
    fn user_files(&self) -> Result<Vec<FilePathBuf>> {
        // Not technically correct but we didn't store this information.
        // Current Uninstaller implementation strips FileType anyway.
        self.user_files
            .list()
            .map(|v| v.into_iter().map(FilePathBuf::Symlink).collect())
    }
    fn clear_user_files(&mut self) -> Result<()> {
        self.user_files.clear()
    }
    fn state_files(&self) -> Result<Vec<FilePathBuf>> {
        // Not technically correct but we didn't store this information.
        // Current Uninstaller implementation strips FileType anyway.
        self.state_files
            .list()
            .map(|v| v.into_iter().map(FilePathBuf::Symlink).collect())
    }
    fn clear_state_files(&mut self) -> Result<()> {
        self.state_files.clear()
    }
}
