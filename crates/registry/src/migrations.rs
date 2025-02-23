use std::sync::OnceLock;

use anyhow::{Context, Result};
use rusqlite_migration::{Migrations, M};

use super::connection::AppConnection;

pub(crate) struct MigrationsConfig {
    migrations: Migrations<'static>,
    stable_version: usize,
    #[cfg(test)]
    rolled_back_version: usize,
}

impl MigrationsConfig {
    pub(crate) fn to_stable(&self, conn: &mut AppConnection) -> Result<()> {
        self.migrations
            .to_version(conn.as_mut(), self.stable_version)
            .with_context(|| {
                format!(
                    "failed to migrate to stable version {}",
                    self.stable_version
                )
            })
    }
    #[cfg(test)]
    pub(crate) fn to_rolled_back(&self, conn: &mut AppConnection) -> Result<()> {
        self.migrations
            .to_version(conn.as_mut(), self.rolled_back_version)
            .with_context(|| {
                format!(
                    "failed to migrate to rolled back version {}",
                    self.rolled_back_version
                )
            })
    }
}

pub(crate) fn config() -> &'static MigrationsConfig {
    static INSTANCE: OnceLock<MigrationsConfig> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        // Migrations must never change their index.
        // Migrations must end with a semicolon.
        let stable: Vec<M> = vec![
            // This Vec is append-only.
            // If there is an issue with a migration add another one.
            M::up(
                "
                CREATE TABLE files (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    purpose INTEGER NOT NULL,
                    file_type INTEGER NOT NULL,
                    path BLOB NOT NULL
                );
                ",
            ),
            M::up(
                // ALTER TABLE files STRICT.
                "
                ALTER TABLE files RENAME TO files_non_strict;
                CREATE TABLE files (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    purpose INTEGER NOT NULL,
                    file_type INTEGER NOT NULL,
                    path BLOB NOT NULL
                ) STRICT;
                INSERT INTO files SELECT * FROM files_non_strict;
                DROP TABLE files_non_strict;
                ",
            )
            .down(
                // ALTER TABLE files NO STRICT.
                "
                CREATE TABLE files_non_strict (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    purpose INTEGER NOT NULL,
                    file_type INTEGER NOT NULL,
                    path BLOB NOT NULL
                );
                INSERT INTO files_non_strict SELECT * FROM files;
                DROP TABLE files;
                ALTER TABLE files_non_strict RENAME TO files;
                ",
            ),
            M::up(
                "
                CREATE TABLE updates (
                    id INTEGER PRIMARY KEY AUTOINCREMENT
                ) STRICT;
                ",
            )
            .down("DROP TABLE updates;"),
            M::up(
                "
                ALTER TABLE files
                ADD COLUMN update_id INTEGER
                    DEFAULT (NULL)
                    REFERENCES updates (id)
                    ON DELETE CASCADE
                ",
            )
            .down(
                "
                ALTER TABLE files
                DROP COLUMN update_id;
                ",
            ),
        ];
        let rolled_back: Vec<M> = vec![
            // This Vec can be modified.
            // Move migrations from stable here to roll them back.
            // Only works reliably on a single machine due to an unknown
            // distribution propagation, so only practical for development.
        ];
        let stable_size = stable.len();
        #[cfg(test)]
        let rolled_back_size = rolled_back.len();
        MigrationsConfig {
            migrations: Migrations::new([stable, rolled_back].concat()),
            stable_version: stable_size,
            #[cfg(test)]
            rolled_back_version: stable_size + rolled_back_size,
        }
    })
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_migrations_empty_to_stable() {
        let mut conn = AppConnection::open_in_memory().unwrap();

        config().to_stable(&mut conn).expect("must be ok");
    }

    #[test]
    fn test_migrations_empty_to_rolled_back() {
        let mut conn = AppConnection::open_in_memory().unwrap();

        config().to_rolled_back(&mut conn).expect("must be ok");
    }

    #[test]
    fn test_migrations_stable_to_rolled_back() {
        let mut conn = AppConnection::open_in_memory().unwrap();
        config().to_stable(&mut conn).expect("must be ok");

        config().to_rolled_back(&mut conn).expect("must be ok");
    }

    #[test]
    fn test_migrations_rolled_back_to_stable() {
        let mut conn = AppConnection::open_in_memory().unwrap();
        config().to_rolled_back(&mut conn).expect("must be ok");

        config().to_stable(&mut conn).expect("must be ok");
    }

    #[test]
    fn test_migrations_database_too_far_ahead() {
        let mut conn = AppConnection::open_in_memory().unwrap();
        conn.as_ref()
            .pragma_update(None, "user_version", 1000)
            .unwrap();

        let err = config().to_stable(&mut conn).unwrap_err();

        let err = err.downcast::<rusqlite_migration::Error>().unwrap();
        assert_eq!(
            err,
            rusqlite_migration::Error::MigrationDefinition(
                rusqlite_migration::MigrationDefinitionError::DatabaseTooFarAhead
            )
        );
    }
}
