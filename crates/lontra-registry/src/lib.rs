use anyhow::Result;

mod connection;
mod file_type;
mod migrations;
mod model;
pub(crate) mod proxied;
mod queries;
#[cfg(test)]
mod row_queries;
mod runtime;
pub mod sqlite;

pub use file_type::{FilePath, FilePathBuf, FileType};

pub trait Registry {
    fn user_files(&mut self) -> Result<Vec<FilePathBuf>>;
    fn register_user_file(&mut self, file: FilePath) -> Result<()>;
    fn clear_user_files(&mut self) -> Result<()>;

    fn state_files(&mut self) -> Result<Vec<FilePathBuf>>;
    fn register_state_file(&mut self, file: FilePath) -> Result<()>;
    fn clear_state_files(&mut self) -> Result<()>;

    fn config_get(&mut self, key: &str, default_value: &str) -> Result<String>;
    fn config_set(&mut self, key: &str, value: &str) -> Result<()>;
}
