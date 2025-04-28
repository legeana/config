use anyhow::{Context as _, Result};
use rusqlite::Connection;

use super::connection::AppConnection;
use super::file_type::{self, FilePathBuf};
use super::model::{FilePurpose, SqlPathBuf, UpdateId};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UpdateRow {
    pub id: UpdateId,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FileRow {
    pub update_id: UpdateId,
    pub purpose: FilePurpose,
    pub file: FilePathBuf,
}

pub(crate) trait RowQueries
where
    Self: AsRef<Connection>,
{
    fn update_rows(&self) -> Result<Vec<UpdateRow>> {
        let mut stmt = self
            .as_ref()
            .prepare_cached(
                "
                SELECT id FROM updates
                ORDER BY id ASC
                ",
            )
            .context("failed to prepare statement")?;
        let update_rows: Result<Vec<_>, _> = stmt
            .query_map([], |row| Ok(UpdateRow { id: row.get("id")? }))
            .context("failed to query updates")?
            .collect();
        update_rows.context("query updates")
    }

    fn file_rows(&self) -> Result<Vec<FileRow>> {
        let mut stmt = self
            .as_ref()
            .prepare_cached(
                "
                SELECT update_id, purpose, file_type, path
                FROM files
                ORDER BY id ASC
                ",
            )
            .context("files statement prepare")?;
        let file_rows: Result<Vec<_>, _> = stmt
            .query_map([], |row| {
                let file_type: file_type::Type = row.get("file_type")?;
                let SqlPathBuf(path) = row.get("path")?;
                Ok(FileRow {
                    update_id: row.get("update_id")?,
                    purpose: row.get("purpose")?,
                    file: file_type.with_path_buf(path),
                })
            })
            .context("failed to query files")?
            .collect();
        file_rows.context("query files")
    }
}

impl RowQueries for AppConnection {}
