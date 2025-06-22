use anyhow::{Context as _, Result};
use sqlx::Connection as _;
use sqlx::SqliteConnection;
use sqlx::query;

use super::connection::AppConnection;
use super::file_type::{self, FilePath, FilePathBuf};
use super::model::{FilePurpose, SqlPath, SqlPathBuf, UpdateId};

pub(crate) trait AppQueries
where
    Self: AsMut<SqliteConnection>,
{
    #[allow(dead_code)]
    async fn create_update(&mut self) -> Result<UpdateId> {
        let row_id = query!(
            "
            INSERT INTO updates
            DEFAULT VALUES
            ",
        )
        .execute(self.as_mut())
        .await
        .context("failed to create new update")?
        .last_insert_rowid();
        Ok(UpdateId(row_id))
    }

    #[allow(dead_code)]
    async fn delete_other_updates(&mut self, update: UpdateId) -> Result<()> {
        query!(
            "
            DELETE FROM updates
            WHERE id != ?
            ",
            update,
        )
        .execute(self.as_mut())
        .await
        .with_context(|| format!("failed to delete other {update:?}"))?;
        Ok(())
    }

    async fn register_file(
        &mut self,
        update: Option<UpdateId>,
        purpose: FilePurpose,
        file: FilePath<'_>,
    ) -> Result<()> {
        let file_type = file.file_type();
        let path = SqlPath(file.path());
        query!(
            "
            INSERT INTO files
            (update_id, purpose, file_type, path)
            VALUES (?, ?, ?, ?)
            ",
            update,
            purpose,
            file_type,
            path,
        )
        .execute(self.as_mut())
        .await
        .with_context(|| format!("failed to register {path:?}"))?;
        Ok(())
    }

    async fn files(&mut self, purpose: FilePurpose) -> Result<Vec<FilePathBuf>> {
        let files = query!(
            r#"
            SELECT
                file_type AS "file_type: file_type::Type",
                path AS "path: SqlPathBuf"
            FROM files
            WHERE
                purpose = ?
            ORDER BY id ASC
            "#,
            purpose,
        )
        .fetch_all(self.as_mut())
        .await
        .context("failed to query files")?
        .into_iter()
        .map(|row| {
            let file_type = row.file_type;
            let SqlPathBuf(path) = row.path;
            file_type.with_path_buf(path)
        })
        .collect();
        Ok(files)
    }

    #[allow(dead_code)]
    async fn files_from_other_updates(
        &mut self,
        update: UpdateId,
        purpose: FilePurpose,
    ) -> Result<Vec<FilePathBuf>> {
        let files = query!(
            r#"
            SELECT
                file_type AS "file_type: file_type::Type",
                path AS "path: SqlPathBuf"
            FROM files
            WHERE
                update_id != ? AND
                purpose = ?
            ORDER BY id ASC
            "#,
            update,
            purpose,
        )
        .fetch_all(self.as_mut())
        .await
        .context("failed to query files")?
        .into_iter()
        .map(|row| {
            let file_type = row.file_type;
            let SqlPathBuf(path) = row.path;
            file_type.with_path_buf(path)
        })
        .collect();
        Ok(files)
    }

    async fn clear_files(&mut self, purpose: FilePurpose) -> Result<()> {
        query!(
            "
            DELETE FROM files
            WHERE
                purpose = ?
            ",
            purpose,
        )
        .execute(self.as_mut())
        .await
        .with_context(|| format!("clear {purpose:?} files"))?;
        Ok(())
    }

    async fn config_get(&mut self, key: &str, default_value: &str) -> Result<String> {
        let mut tx = self.as_mut().begin().await?;
        let r = query!(
            "
            SELECT value
            FROM config
            WHERE
                key = ?
            ",
            key,
        )
        .fetch_optional(&mut *tx)
        .await
        .with_context(|| format!("failed to query config for {key:?}"))?;
        if let Some(r) = r {
            return Ok(r.value);
        }
        query!(
            "
            INSERT INTO config (key, value)
            VALUES(?, ?)
            ",
            key,
            default_value,
        )
        .execute(&mut *tx)
        .await
        .with_context(|| {
            format!("failed to set config {key:?}'s value to default {default_value:?}")
        })?;
        tx.commit().await.with_context(|| {
            format!("failed to commit transaction config_get({key:?}, {default_value:?})")
        })?;
        Ok(default_value.to_owned())
    }

    async fn config_set(&mut self, key: &str, value: &str) -> Result<()> {
        query!(
            "
            INSERT INTO config (key, value)
            VALUES(?, ?)
            ON CONFLICT(key)
            DO UPDATE SET value=?
            ",
            key,
            value,
            value,
        )
        .execute(self.as_mut())
        .await
        .with_context(|| format!("failed to set {key:?} to {value:?}"))?;
        Ok(())
    }
}

impl AppQueries for AppConnection {}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use pretty_assertions::assert_eq;
    use test_case::test_case;

    use crate::row_queries::{FileRow, RowQueries as _, UpdateRow};

    use super::*;

    async fn conn() -> AppConnection {
        let mut conn = AppConnection::open_in_memory()
            .await
            .expect("open_in_memory");
        crate::migrations::config()
            .run(conn.as_mut())
            .await
            .expect("migrate to_stable");
        conn
    }

    #[test_case(FilePurpose::User)]
    #[test_case(FilePurpose::State)]
    fn test_register_file(purpose: FilePurpose) {
        crate::runtime::block_on(async {
            let mut conn = conn().await;
            conn.register_file(None, purpose, FilePath::new_symlink("/test/file"))
                .await
                .expect("register_file");

            assert_eq!(
                conn.file_rows().await.unwrap(),
                vec![FileRow {
                    update_id: None,
                    purpose,
                    file: FilePathBuf::new_symlink("/test/file"),
                }],
            );
        });
    }

    #[test_case(FilePurpose::User, FilePurpose::State)]
    #[test_case(FilePurpose::State, FilePurpose::User)]
    fn test_files(purpose: FilePurpose, other_purpose: FilePurpose) {
        crate::runtime::block_on(async {
            let mut conn = conn().await;
            conn.register_file(None, purpose, FilePath::new_symlink("/test/file"))
                .await
                .expect("register_file");
            conn.register_file(
                None,
                other_purpose,
                FilePath::new_symlink("/test/other/file"),
            )
            .await
            .expect("register_file");

            let files = conn.files(purpose).await.unwrap();
            let other_files = conn.files(other_purpose).await.unwrap();

            assert_eq!(files, vec![FilePathBuf::new_symlink("/test/file")]);
            assert_eq!(
                other_files,
                vec![FilePathBuf::new_symlink("/test/other/file")]
            );
        });
    }

    #[test_case(FilePurpose::User)]
    #[test_case(FilePurpose::State)]
    fn test_files_order(purpose: FilePurpose) {
        crate::runtime::block_on(async {
            let mut conn = conn().await;
            let files = vec![
                FilePath::new_symlink("/test/2/file/1"),
                FilePath::new_symlink("/test/1/file/2"),
                FilePath::new_symlink("/test/3/file/3"),
            ];
            for f in files.iter().copied() {
                conn.register_file(None, purpose, f)
                    .await
                    .expect("register_file");
            }

            assert_eq!(files, conn.files(purpose).await.unwrap());
        });
    }

    #[test_case(FilePurpose::User)]
    #[test_case(FilePurpose::State)]
    fn test_files_from_other_updates(purpose: FilePurpose) {
        crate::runtime::block_on(async {
            let mut conn = conn().await;
            let update = conn.create_update().await.expect("create_update");
            let other_update = conn.create_update().await.expect("create_update");
            conn.register_file(Some(update), purpose, FilePath::new_symlink("/this/update"))
                .await
                .expect("register_file");
            conn.register_file(
                Some(other_update),
                purpose,
                FilePath::new_symlink("/other/update"),
            )
            .await
            .expect("register_file");

            let files = conn
                .files_from_other_updates(update, purpose)
                .await
                .expect("files_from_other_updates");

            assert_eq!(files, vec![FilePathBuf::new_symlink("/other/update")]);
        });
    }

    #[test_case(FilePurpose::User)]
    #[test_case(FilePurpose::State)]
    fn test_files_from_other_updates_order(purpose: FilePurpose) {
        crate::runtime::block_on(async {
            let mut conn = conn().await;
            let want_files = vec![
                FilePath::new_symlink("/test/2/file/1"),
                FilePath::new_symlink("/test/1/file/2"),
                FilePath::new_symlink("/test/3/file/3"),
            ];
            let update = conn.create_update().await.expect("create_update");
            let other_update = conn.create_update().await.expect("create_update");
            for f in want_files.iter().copied() {
                conn.register_file(Some(other_update), purpose, f)
                    .await
                    .expect("register_file");
            }

            let got_files = conn
                .files_from_other_updates(update, purpose)
                .await
                .expect("files_from_other_updates");

            assert_eq!(got_files, want_files);
        });
    }

    #[test_case(FilePurpose::User)]
    #[test_case(FilePurpose::State)]
    fn test_clear_files(purpose: FilePurpose) {
        crate::runtime::block_on(async {
            let mut conn = conn().await;
            conn.register_file(None, purpose, FilePath::new_symlink("/test/file"))
                .await
                .expect("register_file");
            assert_eq!(
                conn.files(purpose).await.unwrap(),
                vec![FilePathBuf::new_symlink("/test/file")]
            );

            conn.clear_files(purpose).await.expect("clear_files");

            assert_eq!(conn.file_rows().await.unwrap(), vec![]);
        });
    }

    #[test_case(FilePurpose::User, FilePurpose::State)]
    #[test_case(FilePurpose::State, FilePurpose::User)]
    fn test_clear_files_does_not_delete_other_files(
        purpose: FilePurpose,
        other_purpose: FilePurpose,
    ) {
        crate::runtime::block_on(async {
            let mut conn = conn().await;
            conn.register_file(None, purpose, FilePath::new_symlink("/test/file"))
                .await
                .expect("register_file");
            assert_eq!(
                conn.files(purpose).await.unwrap(),
                vec![FilePathBuf::new_symlink("/test/file")]
            );
            conn.register_file(None, other_purpose, FilePath::new_symlink("/other/file"))
                .await
                .expect("register_file");
            assert_eq!(
                conn.files(other_purpose).await.unwrap(),
                vec![FilePathBuf::new_symlink("/other/file")]
            );

            conn.clear_files(purpose).await.expect("clear_files");

            assert_eq!(
                conn.file_rows().await.unwrap(),
                vec![FileRow {
                    update_id: None,
                    purpose: other_purpose,
                    file: FilePathBuf::new_symlink("/other/file"),
                }],
            );
        });
    }

    #[tokio::test]
    async fn test_create_update() {
        let mut conn = conn().await;

        let update = conn.create_update().await.expect("create_update");

        let updates = conn.update_rows().await.expect("updates");
        assert_eq!(updates, vec![UpdateRow { id: update }]);
    }

    #[tokio::test]
    async fn test_delete_other_updates() {
        let mut conn = conn().await;
        let update = conn.create_update().await.expect("create_update");
        let other_update = conn.create_update().await.expect("create_update");
        assert_eq!(
            conn.update_rows().await.unwrap(),
            vec![UpdateRow { id: update }, UpdateRow { id: other_update }]
        );

        conn.delete_other_updates(update)
            .await
            .expect("delete_update");

        assert_eq!(
            conn.update_rows().await.unwrap(),
            vec![UpdateRow { id: update }]
        );
    }

    #[tokio::test]
    async fn test_delete_files_via_update() {
        let mut conn = conn().await;
        let update = conn.create_update().await.expect("create_update");
        let other_update = conn.create_update().await.expect("create_update");
        conn.register_file(
            Some(update),
            FilePurpose::User,
            FilePath::new_symlink("test-update"),
        )
        .await
        .expect("register_file");
        conn.register_file(
            Some(other_update),
            FilePurpose::User,
            FilePath::new_symlink("test-other-update"),
        )
        .await
        .expect("register_file");

        conn.delete_other_updates(update)
            .await
            .expect("delete_other_updates");

        assert_eq!(
            conn.file_rows().await.expect("file_rows"),
            vec![FileRow {
                update_id: Some(update),
                purpose: FilePurpose::User,
                file: FilePathBuf::new_symlink("test-update"),
            }],
        );
    }

    #[tokio::test]
    async fn test_config_get_uninitialised() {
        let mut conn = conn().await;

        let value = conn
            .config_get("new-key", "default-value")
            .await
            .expect("config_get");

        assert_eq!(value, "default-value");
        assert_eq!(
            conn.config_rows().await.unwrap(),
            HashMap::from([("new-key".to_owned(), "default-value".to_owned())])
        );
    }

    #[tokio::test]
    async fn test_config_get_initialised() {
        let mut conn = conn().await;
        conn.config_set("key", "starting-value").await.unwrap();
        assert_eq!(
            conn.config_rows().await.unwrap(),
            HashMap::from([("key".to_owned(), "starting-value".to_owned())])
        );

        let value = conn
            .config_get("key", "default-value")
            .await
            .expect("config_get");

        assert_eq!(value, "starting-value");
        assert_eq!(
            conn.config_rows().await.unwrap(),
            HashMap::from([("key".to_owned(), "starting-value".to_owned())])
        );
    }

    #[tokio::test]
    async fn test_config_set_uninitialised() {
        let mut conn = conn().await;

        conn.config_set("key", "value").await.expect("config_get");

        assert_eq!(
            conn.config_rows().await.unwrap(),
            HashMap::from([("key".to_owned(), "value".to_owned())])
        );
    }

    #[tokio::test]
    async fn test_config_set_initialised() {
        let mut conn = conn().await;
        conn.config_set("key", "starting-value").await.unwrap();
        assert_eq!(
            conn.config_rows().await.unwrap(),
            HashMap::from([("key".to_owned(), "starting-value".to_owned())])
        );

        conn.config_set("key", "value").await.expect("config_get");

        assert_eq!(
            conn.config_rows().await.unwrap(),
            HashMap::from([("key".to_owned(), "value".to_owned())])
        );
    }
}
