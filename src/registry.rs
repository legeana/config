use std::path::Path;

use anyhow::Result;

pub trait Registry {
    fn register_symlink(&mut self, path: &Path) -> Result<()>;
}
