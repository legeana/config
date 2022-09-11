use std::path::{Path, PathBuf};

use anyhow::Result;

pub trait Registry {
    fn register(&mut self, path: &Path) -> Result<()>;
    fn paths(&self) -> Result<Vec<PathBuf>>;
    fn clear(&mut self) -> Result<()>;
}
