use std::path::Path;

use anyhow::{Context as _, Result, anyhow};
use sqlx::Connection as _;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::sqlite::SqliteConnection;

const APPLICATION_ID: i32 = 0x12fe_0c02;

#[derive(Debug)]
pub(crate) struct AppConnection(SqliteConnection);

impl AsRef<SqliteConnection> for AppConnection {
    fn as_ref(&self) -> &SqliteConnection {
        &self.0
    }
}

impl AsMut<SqliteConnection> for AppConnection {
    fn as_mut(&mut self) -> &mut SqliteConnection {
        &mut self.0
    }
}

impl AppConnection {
    #[cfg(test)]
    pub(crate) async fn open_in_memory() -> Result<Self> {
        let opts = default_options().in_memory(true);
        let conn = SqliteConnection::connect_with(&opts)
            .await
            .context("failed to open in memory")?;
        Self::with_raw(conn).await
    }

    pub(crate) async fn open(path: &Path) -> Result<Self> {
        let opts = default_options().filename(path);
        let conn = SqliteConnection::connect_with(&opts)
            .await
            .with_context(|| format!("failed to open {path:?}"))?;
        Self::with_raw(conn)
            .await
            .with_context(|| format!("failed to initialise {path:?}"))
    }

    async fn with_raw(mut conn: SqliteConnection) -> Result<Self> {
        init_app_id(&mut conn).await?;
        Ok(Self(conn))
    }

    pub(crate) async fn close(self) -> Result<()> {
        self.0.close().await.context("failed to close connection")
    }
}

async fn query_app_id(conn: &mut SqliteConnection) -> Result<i32> {
    sqlx::query_scalar("PRAGMA application_id")
        .fetch_one(conn)
        .await
        .context("PRAGMA application_id")
}

async fn init_app_id(conn: &mut SqliteConnection) -> Result<()> {
    let mut app_id: i32 = query_app_id(conn).await?;
    if app_id == 0 {
        // Not initialised.
        let query = format!("PRAGMA application_id = {APPLICATION_ID}");
        sqlx::query(&query)
            .execute(&mut *conn)
            .await
            .context(query)?;
        app_id = query_app_id(conn).await?;
    }
    if app_id != APPLICATION_ID {
        return Err(anyhow!(
            "unexpected application_id {app_id:x}, want {APPLICATION_ID:x}"
        ));
    }
    Ok(())
}

fn default_options() -> SqliteConnectOptions {
    use sqlx::sqlite;

    let mut opts = SqliteConnectOptions::new();

    // Performance.
    // https://www.sqlite.org/pragma.html#pragma_synchronous
    opts = opts.synchronous(sqlite::SqliteSynchronous::Normal);
    // https://www.sqlite.org/pragma.html#pragma_journal_mode
    opts = opts.journal_mode(sqlite::SqliteJournalMode::Wal);

    // Behaviour.
    // https://www.sqlite.org/pragma.html#pragma_foreign_keys
    opts = opts.foreign_keys(true);

    // https://www.sqlite.org/c3ref/open.html
    // SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE
    opts = opts.create_if_missing(true);

    opts
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn test_application_id_create() {
        let mut app_conn = AppConnection::open_in_memory()
            .await
            .expect("open_in_memory");

        let id: i32 = query_app_id(app_conn.as_mut()).await.unwrap();

        assert_eq!(id, APPLICATION_ID);
    }

    async fn open_with_app_id(app_id: i32) -> SqliteConnection {
        let opts = default_options()
            .in_memory(true)
            .pragma("application_id", format!("{app_id}"));
        let mut conn = SqliteConnection::connect_with(&opts)
            .await
            .expect("open_in_memory");
        let id: i32 = query_app_id(&mut conn).await.unwrap();
        assert_eq!(id, app_id);
        conn
    }

    #[tokio::test]
    async fn test_application_id_matching() {
        let conn = open_with_app_id(APPLICATION_ID).await;

        let mut app_conn = AppConnection::with_raw(conn).await.unwrap();

        let id: i32 = query_app_id(app_conn.as_mut()).await.unwrap();
        assert_eq!(id, APPLICATION_ID);
    }

    #[tokio::test]
    async fn test_application_id_not_matching() {
        let conn = open_with_app_id(123).await;

        let err = AppConnection::with_raw(conn).await.unwrap_err();

        assert_eq!(
            err.to_string(),
            "unexpected application_id 7b, want 12fe0c02"
        );
    }
}
