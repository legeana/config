use std::sync::OnceLock;

use anyhow::{Context, Result};
use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

pub(super) struct MigrationsConfig {
    migrations: Migrations<'static>,
    stable_version: usize,
    #[allow(dead_code)]
    unstable_version: usize,
}

impl MigrationsConfig {
    pub fn to_stable(&self, conn: &mut Connection) -> Result<()> {
        self.migrations
            .to_version(conn, self.stable_version)
            .with_context(|| {
                format!(
                    "failed to migrate to stable version {}",
                    self.stable_version
                )
            })
    }
    #[allow(dead_code)]
    pub fn to_unstable(&self, conn: &mut Connection) -> Result<()> {
        self.migrations
            .to_version(conn, self.unstable_version)
            .with_context(|| {
                format!(
                    "failed to migrate to unstable version {}",
                    self.unstable_version
                )
            })
    }
}

pub(super) fn config() -> &'static MigrationsConfig {
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
        ];
        let unstable: Vec<M> = vec![
            // This Vec can be modified.
            // Used for experimental changes that may be reverted.
        ];
        let stable_size = stable.len();
        let unstable_size = unstable.len();
        MigrationsConfig {
            migrations: Migrations::new([stable, unstable].concat()),
            stable_version: stable_size,
            unstable_version: stable_size + unstable_size,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations_empty_to_stable() {
        let mut conn = Connection::open_in_memory().unwrap();

        config().to_stable(&mut conn).expect("must be ok");
    }

    #[test]
    fn test_migrations_empty_to_unstable() {
        let mut conn = Connection::open_in_memory().unwrap();

        config().to_unstable(&mut conn).expect("must be ok");
    }

    #[test]
    fn test_migrations_stable_to_unstable() {
        let mut conn = Connection::open_in_memory().unwrap();
        config().to_stable(&mut conn).expect("must be ok");

        config().to_unstable(&mut conn).expect("must be ok");
    }

    #[test]
    fn test_migrations_unstable_to_stable() {
        let mut conn = Connection::open_in_memory().unwrap();
        config().to_unstable(&mut conn).expect("must be ok");

        config().to_stable(&mut conn).expect("must be ok");
    }
}
