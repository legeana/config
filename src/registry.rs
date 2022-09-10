use std::{
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};

pub trait Registry {
    fn register(&mut self, path: &Path) -> Result<()>;
    fn paths(&self) -> Result<Vec<PathBuf>>;
    fn clear(&mut self) -> Result<()>;
}

pub struct FileRegistry {
    path: PathBuf,
}

impl FileRegistry {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Registry for FileRegistry {
    fn register(&mut self, path: &Path) -> Result<()> {
        let mut paths = self.paths()?;
        paths.push(path.to_owned());
        let db = std::fs::File::create(&self.path)
            .with_context(|| format!("unable to open {}", self.path.display()))?;
        let mut writer = std::io::BufWriter::new(db);
        for path in paths {
            let s = path
                .to_str()
                .ok_or(anyhow!("{} is not a valid unicode", path.display()))?;
            write!(&mut writer, "{}\n", s)
                .with_context(|| format!("failed to write to {}", self.path.display()))?;
        }
        return writer
            .flush()
            .with_context(|| format!("unable to write to {}", self.path.display()));
    }
    fn paths(&self) -> Result<Vec<PathBuf>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        if !self.path.is_file() {
            return Err(anyhow!("{} registry must be a file", self.path.display()));
        }
        let data = std::fs::read_to_string(&self.path)
            .with_context(|| format!("failed to read {}", self.path.display()))?;
        return Ok(data
            .split('\n')
            .filter(|s| !s.is_empty())
            .map(|s| PathBuf::from(s))
            .collect());
    }
    fn clear(&mut self) -> Result<()> {
        std::fs::remove_file(&self.path)
            .with_context(|| format!("failed to remove {}", self.path.display()))
    }
}
