use anyhow::{Context, Result};
use rusqlite::{named_params, Connection};

use super::connection::AppConnection;
use super::file_type::{self, FilePath, FilePathBuf};
use super::model::{FilePurpose, SqlPath, SqlPathBuf, UpdateId};

pub(super) trait AppQueries
where
    Self: AsRef<Connection>,
{
    #[allow(dead_code)]
    fn create_update(&self) -> Result<UpdateId> {
        let mut stmt = self
            .as_ref()
            .prepare_cached(
                "
                INSERT INTO updates
                DEFAULT VALUES
                ",
            )
            .context("failed to prepare statement")?;
        let row_id = stmt.insert([]).context("failed to create new update")?;
        Ok(UpdateId(Some(row_id)))
    }

    #[allow(dead_code)]
    fn delete_other_updates(&self, update: UpdateId) -> Result<()> {
        let mut stmt = self
            .as_ref()
            .prepare_cached(
                "
                DELETE FROM updates
                WHERE id != :id
                ",
            )
            .context("failed to prepare statement")?;
        stmt.execute(named_params![":id": update])
            .with_context(|| format!("failed to delete other {update:?}"))?;
        Ok(())
    }

    fn register_file(&self, update: UpdateId, purpose: FilePurpose, file: FilePath) -> Result<()> {
        let file_type = file.file_type();
        let path = SqlPath(file.path());
        let mut stmt = self
            .as_ref()
            .prepare_cached(
                "
                INSERT INTO files
                (update_id, purpose, file_type, path)
                VALUES (:update, :purpose, :file_type, :path)
                ",
            )
            .context("failed to prepare statement")?;
        stmt.execute(named_params![
            ":update": update,
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
                let file_type: file_type::Type = row.get("file_type")?;
                let SqlPathBuf(path) = row.get("path")?;
                Ok(file_type.with_path_buf(path))
            })
            .context("failed to query files")?
            .collect();
        files.context("query files")
    }

    #[allow(dead_code)]
    fn files_from_other_updates(
        &self,
        update: UpdateId,
        purpose: FilePurpose,
    ) -> Result<Vec<FilePathBuf>> {
        let mut stmt = self
            .as_ref()
            .prepare_cached(
                "
                SELECT file_type, path
                FROM files
                WHERE
                    update_id != :update AND
                    purpose = :purpose
                ORDER BY id ASC
                ",
            )
            .context("files statement prepare")?;
        let files: Result<Vec<_>, _> = stmt
            .query_map(
                named_params![":update": update, ":purpose": purpose],
                |row| {
                    let file_type: file_type::Type = row.get("file_type")?;
                    let SqlPathBuf(path) = row.get("path")?;
                    Ok(file_type.with_path_buf(path))
                },
            )
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

    use crate::registry::row_queries::{FileRow, RowQueries, UpdateRow};

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
    fn test_register_file(conn: AppConnection, purpose: FilePurpose, _other_purpose: FilePurpose) {
        conn.register_file(UpdateId(None), purpose, FilePath::new_symlink("/test/file"))
            .expect("register_file");

        assert_eq!(
            conn.file_rows().unwrap(),
            vec![FileRow {
                update_id: UpdateId(None),
                purpose,
                file: FilePathBuf::new_symlink("/test/file"),
            }],
        );
    }

    #[apply(sqlite_registry_test)]
    fn test_files(conn: AppConnection, purpose: FilePurpose, other_purpose: FilePurpose) {
        conn.register_file(UpdateId(None), purpose, FilePath::new_symlink("/test/file"))
            .expect("register_file");
        conn.register_file(
            UpdateId(None),
            other_purpose,
            FilePath::new_symlink("/test/other/file"),
        )
        .expect("register_file");

        let files = conn.files(purpose).unwrap();
        let other_files = conn.files(other_purpose).unwrap();

        assert_eq!(files, vec![FilePathBuf::new_symlink("/test/file")]);
        assert_eq!(
            other_files,
            vec![FilePathBuf::new_symlink("/test/other/file")]
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
            conn.register_file(UpdateId(None), purpose, f)
                .expect("register_file");
        }

        assert_eq!(files, conn.files(purpose).unwrap());
    }

    #[apply(sqlite_registry_test)]
    fn test_files_from_other_updates(
        conn: AppConnection,
        purpose: FilePurpose,
        _other_purpose: FilePurpose,
    ) {
        let update = conn.create_update().expect("create_update");
        let other_update = conn.create_update().expect("create_update");
        conn.register_file(update, purpose, FilePath::new_symlink("/this/update"))
            .expect("register_file");
        conn.register_file(
            other_update,
            purpose,
            FilePath::new_symlink("/other/update"),
        )
        .expect("register_file");

        let files = conn
            .files_from_other_updates(update, purpose)
            .expect("files_from_other_updates");

        assert_eq!(files, vec![FilePathBuf::new_symlink("/other/update")]);
    }

    #[apply(sqlite_registry_test)]
    fn test_files_from_other_updates_order(
        conn: AppConnection,
        purpose: FilePurpose,
        _other_purpose: FilePurpose,
    ) {
        let want_files = vec![
            FilePath::new_symlink("/test/2/file/1"),
            FilePath::new_symlink("/test/1/file/2"),
            FilePath::new_symlink("/test/3/file/3"),
        ];
        let update = conn.create_update().expect("create_update");
        let other_update = conn.create_update().expect("create_update");
        for f in want_files.iter().copied() {
            conn.register_file(other_update, purpose, f)
                .expect("register_file");
        }

        let got_files = conn
            .files_from_other_updates(update, purpose)
            .expect("files_from_other_updates");

        assert_eq!(got_files, want_files);
    }

    #[apply(sqlite_registry_test)]
    fn test_clear_files(conn: AppConnection, purpose: FilePurpose, _other_purpose: FilePurpose) {
        conn.register_file(UpdateId(None), purpose, FilePath::new_symlink("/test/file"))
            .expect("register_file");
        assert_eq!(
            conn.files(purpose).unwrap(),
            vec![FilePathBuf::new_symlink("/test/file")]
        );

        conn.clear_files(purpose).expect("clear_files");

        assert_eq!(conn.file_rows().unwrap(), vec![]);
    }

    #[apply(sqlite_registry_test)]
    fn test_clear_files_does_not_delete_other_files(
        conn: AppConnection,
        purpose: FilePurpose,
        other_purpose: FilePurpose,
    ) {
        conn.register_file(UpdateId(None), purpose, FilePath::new_symlink("/test/file"))
            .expect("register_file");
        assert_eq!(
            conn.files(purpose).unwrap(),
            vec![FilePathBuf::new_symlink("/test/file")]
        );
        conn.register_file(
            UpdateId(None),
            other_purpose,
            FilePath::new_symlink("/other/file"),
        )
        .expect("register_file");
        assert_eq!(
            conn.files(other_purpose).unwrap(),
            vec![FilePathBuf::new_symlink("/other/file")]
        );

        conn.clear_files(purpose).expect("clear_files");

        assert_eq!(
            conn.file_rows().unwrap(),
            vec![FileRow {
                update_id: UpdateId(None),
                purpose: other_purpose,
                file: FilePathBuf::new_symlink("/other/file"),
            }],
        );
    }

    #[rstest]
    fn test_create_update(conn: AppConnection) {
        let update = conn.create_update().expect("create_update");

        let updates = conn.update_rows().expect("updates");
        assert_eq!(updates, vec![UpdateRow { id: update }]);
    }

    #[rstest]
    fn test_delete_other_updates(conn: AppConnection) {
        let update = conn.create_update().expect("create_update");
        let other_update = conn.create_update().expect("create_update");
        assert_eq!(
            conn.update_rows().unwrap(),
            vec![UpdateRow { id: update }, UpdateRow { id: other_update }]
        );

        conn.delete_other_updates(update).expect("delete_update");

        assert_eq!(conn.update_rows().unwrap(), vec![UpdateRow { id: update }]);
    }

    #[rstest]
    fn test_delete_files_via_update(conn: AppConnection) {
        let update = conn.create_update().expect("create_update");
        let other_update = conn.create_update().expect("create_update");
        conn.register_file(
            update,
            FilePurpose::User,
            FilePath::new_symlink("test-update"),
        )
        .expect("register_file");
        conn.register_file(
            other_update,
            FilePurpose::User,
            FilePath::new_symlink("test-other-update"),
        )
        .expect("register_file");

        conn.delete_other_updates(update)
            .expect("delete_other_updates");

        assert_eq!(
            conn.file_rows().expect("file_rows"),
            vec![FileRow {
                update_id: update,
                purpose: FilePurpose::User,
                file: FilePathBuf::new_symlink("test-update"),
            }],
        );
    }
}
