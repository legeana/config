use crate::registry;

use std::{
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};

pub struct FileRegistry {
    path: PathBuf,
}

impl FileRegistry {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl registry::Registry for FileRegistry {
    fn register(&mut self, path: &Path) -> Result<()> {
        let mut paths = self.paths()?;
        paths.push(path.to_owned());
        let db = std::fs::File::create(&self.path)
            .with_context(|| format!("unable to open {:?}", self.path))?;
        let mut writer = std::io::BufWriter::new(db);
        for path in paths {
            let s = path
                .to_str()
                .ok_or_else(|| anyhow!("{path:?} is not a valid unicode"))?;
            writeln!(&mut writer, "{}", s)
                .with_context(|| format!("failed to write to {:?}", self.path))?;
        }
        writer
            .flush()
            .with_context(|| format!("unable to write to {:?}", self.path))
    }
    fn paths(&self) -> Result<Vec<PathBuf>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        if !self.path.is_file() {
            return Err(anyhow!("{:?} registry must be a file", self.path));
        }
        let data = std::fs::read_to_string(&self.path)
            .with_context(|| format!("failed to read {:?}", self.path))?;
        Ok(data
            .split('\n')
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .collect())
    }
    fn clear(&mut self) -> Result<()> {
        match std::fs::remove_file(&self.path) {
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
            r => r.with_context(|| format!("failed to remove {:?}", self.path)),
        }
    }
}
