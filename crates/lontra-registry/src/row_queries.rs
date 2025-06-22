use std::collections::HashMap;

use anyhow::{Context as _, Result};
use sqlx::{SqliteConnection, query};

use super::connection::AppConnection;
use super::file_type::{self, FilePathBuf};
use super::model::{FilePurpose, SqlPathBuf, UpdateId};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UpdateRow {
    pub id: UpdateId,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FileRow {
    pub update_id: Option<UpdateId>,
    pub purpose: FilePurpose,
    pub file: FilePathBuf,
}

pub(crate) trait RowQueries
where
    Self: AsMut<SqliteConnection>,
{
    async fn update_rows(&mut self) -> Result<Vec<UpdateRow>> {
        let updates = query!(
            r#"
            SELECT
                id AS "id: UpdateId"
            FROM updates
            ORDER BY id ASC
            "#,
        )
        .fetch_all(self.as_mut())
        .await
        .context("failed to query updates")?
        .into_iter()
        .map(|row| UpdateRow { id: row.id })
        .collect();
        Ok(updates)
    }

    async fn file_rows(&mut self) -> Result<Vec<FileRow>> {
        let files = query!(
            r#"
            SELECT
                update_id AS "update_id: UpdateId",
                purpose AS "purpose: FilePurpose",
                file_type AS "file_type: file_type::Type",
                path AS "path: SqlPathBuf"
            FROM files
            ORDER BY id ASC
            "#,
        )
        .fetch_all(self.as_mut())
        .await
        .context("failed to query files")?
        .into_iter()
        .map(|row| {
            let SqlPathBuf(path) = row.path;
            FileRow {
                update_id: row.update_id,
                purpose: row.purpose,
                file: row.file_type.with_path_buf(path),
            }
        })
        .collect();
        Ok(files)
    }

    async fn config_rows(&mut self) -> Result<HashMap<String, String>> {
        let entries: HashMap<String, String> = query!(
            "
            SELECT key, value
            FROM config
            ",
        )
        .fetch_all(self.as_mut())
        .await
        .context("failed to query config entries")?
        .into_iter()
        .map(|row| (row.key, row.value))
        .collect();
        Ok(entries)
    }
}

impl RowQueries for AppConnection {}
