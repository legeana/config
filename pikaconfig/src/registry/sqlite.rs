use std::path::Path;

use anyhow::{Context, Result};

use super::connection::AppConnection;
use super::model::{FilePurpose, UpdateId};
use super::queries::AppQueries;
use super::{FilePath, FilePathBuf, ImmutableRegistry, Registry};

#[derive(Debug)]
pub struct SqliteRegistry {
    conn: AppConnection,
}

impl SqliteRegistry {
    #[cfg(test)]
    fn open_in_memory() -> Result<Self> {
        Self::with_connection(AppConnection::open_in_memory()?)
    }

    pub fn open(path: &Path) -> Result<Self> {
        Self::with_connection(AppConnection::open(path)?)
    }

    fn with_connection(mut conn: AppConnection) -> Result<Self> {
        super::migrations::config()
            .to_stable(&mut conn)
            .context("failed to migrate")?;
        Ok(Self { conn })
    }

    pub fn close(self) -> Result<()> {
        self.conn.close()
    }
}

impl ImmutableRegistry for SqliteRegistry {
    fn user_files(&self) -> Result<Vec<FilePathBuf>> {
        self.conn.files(FilePurpose::User)
    }
    fn clear_user_files(&mut self) -> Result<()> {
        self.conn.clear_files(FilePurpose::User)
    }

    fn state_files(&self) -> Result<Vec<FilePathBuf>> {
        self.conn.files(FilePurpose::State)
    }
    fn clear_state_files(&mut self) -> Result<()> {
        self.conn.clear_files(FilePurpose::State)
    }
}

impl Registry for SqliteRegistry {
    fn register_user_file(&mut self, file: FilePath) -> Result<()> {
        self.conn
            .register_file(UpdateId(None), FilePurpose::User, file)
    }
    fn register_state_file(&mut self, file: FilePath) -> Result<()> {
        self.conn
            .register_file(UpdateId(None), FilePurpose::State, file)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn reg() -> SqliteRegistry {
        SqliteRegistry::open_in_memory().expect("open_in_memory")
    }

    #[test]
    fn test_user_files_register() {
        let mut reg = reg();

        reg.register_user_file(FilePath::new_symlink("test"))
            .expect("register_user_file");

        assert_eq!(
            reg.user_files().unwrap(),
            vec![FilePath::new_symlink("test")]
        );
    }

    #[test]
    fn test_user_files_clear() {
        let mut reg = reg();
        reg.register_user_file(FilePath::new_symlink("test"))
            .expect("register_user_file");

        reg.clear_user_files().expect("clear_user_files");

        assert_eq!(reg.user_files().unwrap(), Vec::<FilePathBuf>::new());
    }

    #[test]
    fn test_state_files_register() {
        let mut reg = reg();
        reg.register_state_file(FilePath::new_symlink("test"))
            .expect("register_state_file");

        assert_eq!(
            reg.state_files().unwrap(),
            vec![FilePath::new_symlink("test")]
        );
    }

    #[test]
    fn test_state_files_clear() {
        let mut reg = reg();
        reg.register_state_file(FilePath::new_symlink("test"))
            .expect("register_state_file");

        reg.clear_state_files().expect("clear_state_files");

        assert_eq!(reg.state_files().unwrap(), Vec::<FilePathBuf>::new());
    }
}
