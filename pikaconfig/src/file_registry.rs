use crate::registry::{FileType, Registry};

use std::{
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};

struct FileList(PathBuf);

impl FileList {
    fn push(&mut self, path: &Path) -> Result<()> {
        let mut paths = self.list()?;
        paths.push(path.to_owned());
        let db = std::fs::File::create(&self.0)
            .with_context(|| format!("unable to open {:?}", self.0))?;
        let mut writer = std::io::BufWriter::new(db);
        for path in paths {
            let s = path
                .to_str()
                .ok_or_else(|| anyhow!("{path:?} is not a valid unicode"))?;
            writeln!(&mut writer, "{}", s)
                .with_context(|| format!("failed to write to {:?}", self.0))?;
        }
        writer
            .flush()
            .with_context(|| format!("unable to write to {:?}", self.0))
    }
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

impl Registry for FileRegistry {
    fn register_user_file(&mut self, path: &Path, _file_type: FileType) -> Result<()> {
        self.user_files.push(path)
    }
    fn user_files(&self) -> Result<Vec<PathBuf>> {
        self.user_files.list()
    }
    fn clear_user_files(&mut self) -> Result<()> {
        self.user_files.clear()
    }
    fn register_state_file(&mut self, path: &Path, _file_type: FileType) -> Result<()> {
        self.state_files.push(path)
    }
    fn state_files(&self) -> Result<Vec<PathBuf>> {
        self.state_files.list()
    }
    fn clear_state_files(&mut self) -> Result<()> {
        self.state_files.clear()
    }
}
