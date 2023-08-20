use std::path::{Path, PathBuf};

use anyhow::Result;

pub enum FileType {
    Symlink,
    Directory,
}

pub trait Registry {
    fn register_user_file(&mut self, path: &Path, file_type: FileType) -> Result<()>;
    fn user_files(&self) -> Result<Vec<PathBuf>>;
    fn clear_user_files(&mut self) -> Result<()>;
}
