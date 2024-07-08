use std::path::{Path, PathBuf};

use anyhow::Result;

pub enum FileType<T> {
    Symlink(T),
    Directory(T),
}

impl<T> FileType<T>
where
    T: AsRef<Path>,
{
    pub fn path(&self) -> &Path {
        match self {
            Self::Symlink(p) => p.as_ref(),
            Self::Directory(p) => p.as_ref(),
        }
    }
}

pub type FilePath<'a> = FileType<&'a Path>;
pub type FilePathBuf = FileType<PathBuf>;

pub trait Registry {
    fn register_user_file(&mut self, file: FilePath) -> Result<()>;
    fn user_files(&self) -> Result<Vec<FilePathBuf>>;
    fn clear_user_files(&mut self) -> Result<()>;

    fn register_state_file(&mut self, file: FilePath) -> Result<()>;
    fn state_files(&self) -> Result<Vec<FilePathBuf>>;
    fn clear_state_files(&mut self) -> Result<()>;
}
