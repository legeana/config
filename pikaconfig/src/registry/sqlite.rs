use std::path::Path;

use anyhow::{Context, Result};
use rusqlite::named_params;

use super::connection::AppConnection;
use super::file_type;
use super::model::{FilePurpose, SqlPath, SqlPathBuf};
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

    fn register_file(&mut self, purpose: FilePurpose, file: FilePath) -> Result<()> {
        let file_type = file.file_type();
        let path = SqlPath(file.path());
        let mut stmt = self
            .conn
            .as_ref()
            .prepare_cached(
                "
                INSERT INTO files
                (purpose, file_type, path)
                VALUES (:purpose, :file_type, :path)
                ",
            )
            .context("failed to prepare statement")?;
        stmt.execute(named_params![
            ":purpose": purpose,
            ":file_type": file_type,
            ":path": path,
        ])
        .with_context(|| format!("failed to register {path:?}"))?;
        Ok(())
    }

    fn files(&self, purpose: FilePurpose) -> Result<Vec<FilePathBuf>> {
        let mut stmt = self
            .conn
            .as_ref()
            .prepare_cached(
                "
                SELECT file_type, path
                FROM files
                WHERE
                    purpose = :purpose
                ORDER BY id ASC
                ",
            )
            .context("files statement prepare")?;
        let files: Result<Vec<_>, _> = stmt
            .query_map(named_params![":purpose": purpose], |row| {
                let file_type: file_type::Type = row.get(0)?;
                let SqlPathBuf(path) = row.get(1)?;
                Ok(file_type.with_path_buf(path))
            })
            .context("failed to query files")?
            .collect();
        files.context("query files")
    }

    fn clear_files(&self, purpose: FilePurpose) -> Result<()> {
        self.conn
            .as_ref()
            .execute(
                "
                DELETE FROM files
                WHERE
                    purpose = :purpose
                ",
                named_params![":purpose": purpose],
            )
            .with_context(|| format!("clear {purpose:?} files"))?;
        Ok(())
    }
}

impl ImmutableRegistry for SqliteRegistry {
    fn user_files(&self) -> Result<Vec<FilePathBuf>> {
        self.files(FilePurpose::User)
    }
    fn clear_user_files(&mut self) -> Result<()> {
        self.clear_files(FilePurpose::User)
    }

    fn state_files(&self) -> Result<Vec<FilePathBuf>> {
        self.files(FilePurpose::State)
    }
    fn clear_state_files(&mut self) -> Result<()> {
        self.clear_files(FilePurpose::State)
    }
}

impl Registry for SqliteRegistry {
    fn register_user_file(&mut self, file: FilePath) -> Result<()> {
        self.register_file(FilePurpose::User, file)
    }
    fn register_state_file(&mut self, file: FilePath) -> Result<()> {
        self.register_file(FilePurpose::State, file)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use rstest_reuse::{apply, template};

    use super::*;

    #[template]
    #[rstest]
    #[case(FilePurpose::User, FilePurpose::State)]
    #[case(FilePurpose::State, FilePurpose::User)]
    fn sqlite_registry_test(#[case] purpose: FilePurpose, #[case] other_purpose: FilePurpose) {}

    #[apply(sqlite_registry_test)]
    fn test_register_file(purpose: FilePurpose, other_purpose: FilePurpose) {
        let mut reg = SqliteRegistry::open_in_memory().expect("open_in_memory");

        reg.register_file(purpose, FilePath::new_symlink("/test/file"))
            .expect("register_file");

        assert_eq!(
            reg.files(purpose).unwrap(),
            vec![FilePathBuf::new_symlink("/test/file")]
        );
        assert_eq!(reg.files(other_purpose).unwrap(), Vec::<FilePathBuf>::new());
    }

    #[apply(sqlite_registry_test)]
    fn test_clear_files(purpose: FilePurpose, other_purpose: FilePurpose) {
        let mut reg = SqliteRegistry::open_in_memory().expect("open_in_memory");
        reg.register_file(purpose, FilePath::new_symlink("/test/file"))
            .expect("register_file");
        assert_eq!(
            reg.files(purpose).unwrap(),
            vec![FilePathBuf::new_symlink("/test/file")]
        );

        reg.clear_files(purpose).expect("clear_files");

        assert_eq!(reg.files(purpose).unwrap(), Vec::<FilePathBuf>::new());
        assert_eq!(reg.files(other_purpose).unwrap(), Vec::<FilePathBuf>::new());
    }

    #[apply(sqlite_registry_test)]
    fn test_clear_files_does_not_delete_other_files(
        purpose: FilePurpose,
        other_purpose: FilePurpose,
    ) {
        let mut reg = SqliteRegistry::open_in_memory().expect("open_in_memory");
        reg.register_file(purpose, FilePath::new_symlink("/test/file"))
            .expect("register_file");
        assert_eq!(
            reg.files(purpose).unwrap(),
            vec![FilePathBuf::new_symlink("/test/file")]
        );
        reg.register_file(other_purpose, FilePath::new_symlink("/other/file"))
            .expect("register_file");
        assert_eq!(
            reg.files(other_purpose).unwrap(),
            vec![FilePathBuf::new_symlink("/other/file")]
        );

        reg.clear_files(purpose).expect("clear_files");

        assert_eq!(reg.files(purpose).unwrap(), Vec::<FilePathBuf>::new());
        assert_eq!(
            reg.files(other_purpose).unwrap(),
            vec![FilePathBuf::new_symlink("/other/file")]
        );
    }

    #[apply(sqlite_registry_test)]
    fn test_files_order(purpose: FilePurpose, _other_purpose: FilePurpose) {
        let mut reg = SqliteRegistry::open_in_memory().expect("open_in_memory");
        let files = vec![
            FilePath::new_symlink("/test/2/file/1"),
            FilePath::new_symlink("/test/1/file/2"),
            FilePath::new_symlink("/test/3/file/3"),
        ];

        for f in files.iter().copied() {
            reg.register_file(purpose, f).expect("register_file");
        }

        assert_eq!(files, reg.files(purpose).unwrap());
    }
}
