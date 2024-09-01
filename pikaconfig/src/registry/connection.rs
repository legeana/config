use std::path::Path;

use anyhow::{anyhow, Context, Error, Result};
use rusqlite::Connection;

const APPLICATION_ID: i32 = 0x12fe0c02;

#[derive(Debug)]
pub(super) struct AppConnection(Connection);

impl AsRef<Connection> for AppConnection {
    fn as_ref(&self) -> &Connection {
        &self.0
    }
}

impl AsMut<Connection> for AppConnection {
    fn as_mut(&mut self) -> &mut Connection {
        &mut self.0
    }
}

impl AppConnection {
    #[cfg(test)]
    pub(super) fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().context("failed to open in memory")?;
        Self::with_raw(conn)
    }

    pub(super) fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path).with_context(|| format!("failed to open {path:?}"))?;
        Self::with_raw(conn).with_context(|| format!("failed to initialise {path:?}"))
    }

    fn with_raw(conn: Connection) -> Result<Self> {
        init_app_id(&conn)?;
        configure_connection(&conn)?;
        Ok(Self(conn))
    }

    pub(super) fn close(self) -> Result<()> {
        self.0
            .close()
            .map_err(|(_conn, err)| Error::new(err).context("failed to close connection"))
    }
}

fn query_app_id(conn: &Connection) -> Result<i32> {
    conn.query_row("PRAGMA application_id", [], |row| row.get(0))
        .context("PRAGMA application_id")
}

fn init_app_id(conn: &Connection) -> Result<()> {
    let mut app_id: i32 = query_app_id(conn)?;
    if app_id == 0 {
        // Not initialised.
        conn.pragma_update(None, "application_id", APPLICATION_ID)?;
        app_id = query_app_id(conn)?;
    }
    if app_id != APPLICATION_ID {
        return Err(anyhow!(
            "unexpected application_id {app_id:x}, want {APPLICATION_ID:x}"
        ));
    }
    Ok(())
}

fn configure_connection(conn: &Connection) -> Result<()> {
    // Performance.
    // https://www.sqlite.org/pragma.html#pragma_synchronous
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    // https://www.sqlite.org/pragma.html#pragma_journal_mode
    conn.pragma_update(None, "journal_mode", "WAL")?;

    // Behaviour.
    // https://www.sqlite.org/pragma.html#pragma_foreign_keys
    conn.pragma_update(None, "foreign_keys", "TRUE")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_application_id_create() {
        let app_conn = AppConnection::open_in_memory().expect("open_in_memory");

        let id: i32 = app_conn
            .as_ref()
            .query_row("PRAGMA application_id", [], |row| row.get(0))
            .unwrap();

        assert_eq!(id, APPLICATION_ID);
    }

    #[test]
    fn test_application_id_matching() {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "application_id", APPLICATION_ID)
            .unwrap();

        let app_conn = AppConnection::with_raw(conn).unwrap();

        let id: i32 = app_conn
            .as_ref()
            .query_row("PRAGMA application_id", [], |row| row.get(0))
            .unwrap();
        assert_eq!(id, APPLICATION_ID);
    }

    #[test]
    fn test_application_id_not_matching() {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "application_id", 123).unwrap();

        let err = AppConnection::with_raw(conn).unwrap_err();

        assert_eq!(
            err.to_string(),
            "unexpected application_id 7b, want 12fe0c02"
        );
    }
}
