use anyhow::Result;

mod file_type;
pub mod sqlite;

pub use file_type::{FilePath, FilePathBuf, FileType};

pub trait ImmutableRegistry {
    fn user_files(&self) -> Result<Vec<FilePathBuf>>;
    fn clear_user_files(&mut self) -> Result<()>;

    fn state_files(&self) -> Result<Vec<FilePathBuf>>;
    fn clear_state_files(&mut self) -> Result<()>;
}

pub trait Registry: ImmutableRegistry {
    fn register_user_file(&mut self, file: FilePath) -> Result<()>;
    fn register_state_file(&mut self, file: FilePath) -> Result<()>;
}
