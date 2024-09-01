use anyhow::{Context, Result};
use rusqlite::Connection;

use super::connection::AppConnection;
use super::model::UpdateId;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct UpdateRow {
    pub id: UpdateId,
}

pub(super) trait RowQueries
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
}

impl RowQueries for AppConnection {}
