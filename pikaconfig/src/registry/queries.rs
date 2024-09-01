use anyhow::{Context, Result};
use rusqlite::{named_params, Connection};

use super::connection::AppConnection;
use super::file_type::{self, FilePath, FilePathBuf};
use super::model::{FilePurpose, SqlPath, SqlPathBuf};

pub(super) trait AppQueries
where
    Self: AsRef<Connection>,
{
    fn register_file(&self, purpose: FilePurpose, file: FilePath) -> Result<()> {
        let file_type = file.file_type();
        let path = SqlPath(file.path());
        let mut stmt = self
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
        self.as_ref()
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

impl AppQueries for AppConnection {}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::{fixture, rstest};
    use rstest_reuse::{apply, template};

    use super::*;

    #[fixture]
    fn conn() -> AppConnection {
        let mut conn = AppConnection::open_in_memory().expect("open_in_memory");
        crate::registry::migrations::config()
            .to_stable(&mut conn)
            .expect("migrate to_stable");
        conn
    }

    #[template]
    #[rstest]
    #[case(FilePurpose::User, FilePurpose::State)]
    #[case(FilePurpose::State, FilePurpose::User)]
    fn sqlite_registry_test(#[case] purpose: FilePurpose, #[case] other_purpose: FilePurpose) {}

    #[apply(sqlite_registry_test)]
    fn test_register_file(conn: AppConnection, purpose: FilePurpose, other_purpose: FilePurpose) {
        conn.register_file(purpose, FilePath::new_symlink("/test/file"))
            .expect("register_file");

        assert_eq!(
            conn.files(purpose).unwrap(),
            vec![FilePathBuf::new_symlink("/test/file")]
        );
        assert_eq!(
            conn.files(other_purpose).unwrap(),
            Vec::<FilePathBuf>::new()
        );
    }

    #[apply(sqlite_registry_test)]
    fn test_clear_files(conn: AppConnection, purpose: FilePurpose, other_purpose: FilePurpose) {
        conn.register_file(purpose, FilePath::new_symlink("/test/file"))
            .expect("register_file");
        assert_eq!(
            conn.files(purpose).unwrap(),
            vec![FilePathBuf::new_symlink("/test/file")]
        );

        conn.clear_files(purpose).expect("clear_files");

        assert_eq!(conn.files(purpose).unwrap(), Vec::<FilePathBuf>::new());
        assert_eq!(
            conn.files(other_purpose).unwrap(),
            Vec::<FilePathBuf>::new()
        );
    }

    #[apply(sqlite_registry_test)]
    fn test_clear_files_does_not_delete_other_files(
        conn: AppConnection,
        purpose: FilePurpose,
        other_purpose: FilePurpose,
    ) {
        conn.register_file(purpose, FilePath::new_symlink("/test/file"))
            .expect("register_file");
        assert_eq!(
            conn.files(purpose).unwrap(),
            vec![FilePathBuf::new_symlink("/test/file")]
        );
        conn.register_file(other_purpose, FilePath::new_symlink("/other/file"))
            .expect("register_file");
        assert_eq!(
            conn.files(other_purpose).unwrap(),
            vec![FilePathBuf::new_symlink("/other/file")]
        );

        conn.clear_files(purpose).expect("clear_files");

        assert_eq!(conn.files(purpose).unwrap(), Vec::<FilePathBuf>::new());
        assert_eq!(
            conn.files(other_purpose).unwrap(),
            vec![FilePathBuf::new_symlink("/other/file")]
        );
    }

    #[apply(sqlite_registry_test)]
    fn test_files_order(conn: AppConnection, purpose: FilePurpose, _other_purpose: FilePurpose) {
        let files = vec![
            FilePath::new_symlink("/test/2/file/1"),
            FilePath::new_symlink("/test/1/file/2"),
            FilePath::new_symlink("/test/3/file/3"),
        ];

        for f in files.iter().copied() {
            conn.register_file(purpose, f).expect("register_file");
        }

        assert_eq!(files, conn.files(purpose).unwrap());
    }
}
