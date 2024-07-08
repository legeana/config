use std::path::{Path, PathBuf};

use anyhow::Result;

#[derive(Clone, Debug, Eq, PartialEq)]
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_file_path_debug() {
        let f = FilePath::Symlink(Path::new("test"));
        assert_eq!(format!("{f:?}"), r#"Symlink("test")"#);
    }

    #[test]
    fn test_file_path_buf_debug() {
        let f = FilePathBuf::Symlink(PathBuf::from("test"));
        assert_eq!(format!("{f:?}"), r#"Symlink("test")"#);
    }
}
