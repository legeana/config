use std::path::Path;

use anyhow::{Context as _, Result};

use super::connection::AppConnection;
use super::model::FilePurpose;
use super::queries::AppQueries as _;
use super::{FilePath, FilePathBuf, Registry};
use crate::migrations;
use crate::runtime::Runtime;

#[derive(Debug)]
pub struct SqliteRegistry {
    rt: Runtime,
    conn: AppConnection,
}

impl SqliteRegistry {
    #[cfg(test)]
    fn open_in_memory() -> Result<Self> {
        let rt = Runtime::new_current_thread()?;
        let conn = rt.block_on(AppConnection::open_in_memory())?;
        Self::with_connection(rt, conn)
    }

    pub fn open(path: &Path) -> Result<Self> {
        let rt = Runtime::new_current_thread()?;
        let conn = rt.block_on(AppConnection::open(path))?;
        Self::with_connection(rt, conn)
    }

    fn with_connection(rt: Runtime, mut conn: AppConnection) -> Result<Self> {
        rt.block_on(async {
            migrations::config()
                .run(conn.as_mut())
                .await
                .context("failed to migrate")
        })?;
        Ok(Self { rt, conn })
    }

    pub fn close(self) -> Result<()> {
        self.rt.block_on(self.conn.close())
    }
}

impl Registry for SqliteRegistry {
    fn user_files(&mut self) -> Result<Vec<FilePathBuf>> {
        self.rt
            .block_on(async { self.conn.files(FilePurpose::User).await })
    }
    fn register_user_file(&mut self, file: FilePath) -> Result<()> {
        self.rt
            .block_on(async { self.conn.register_file(None, FilePurpose::User, file).await })
    }
    fn clear_user_files(&mut self) -> Result<()> {
        self.rt
            .block_on(async { self.conn.clear_files(FilePurpose::User).await })
    }

    fn state_files(&mut self) -> Result<Vec<FilePathBuf>> {
        self.rt
            .block_on(async { self.conn.files(FilePurpose::State).await })
    }
    fn register_state_file(&mut self, file: FilePath) -> Result<()> {
        self.rt.block_on(async {
            self.conn
                .register_file(None, FilePurpose::State, file)
                .await
        })
    }
    fn clear_state_files(&mut self) -> Result<()> {
        self.rt
            .block_on(async { self.conn.clear_files(FilePurpose::State).await })
    }

    fn config_get(&mut self, key: &str, default_value: &str) -> Result<String> {
        self.rt
            .block_on(async { self.conn.config_get(key, default_value).await })
    }
    fn config_set(&mut self, key: &str, value: &str) -> Result<()> {
        self.rt
            .block_on(async { self.conn.config_set(key, value).await })
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
