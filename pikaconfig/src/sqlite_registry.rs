use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{anyhow, Context, Error, Result};
use rusqlite::types::Type;
use rusqlite::{named_params, Connection};
use rusqlite_migration::{Migrations, M};

use crate::registry::{FilePath, FilePathBuf, Registry};

const APPLICATION_ID: i32 = 0x12fe0c02;

struct MigrationsConfig {
    migrations: Migrations<'static>,
    stable_version: usize,
    #[allow(dead_code)]
    unstable_version: usize,
}

impl MigrationsConfig {
    fn to_stable(&self, conn: &mut Connection) -> Result<()> {
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
    fn to_unstable(&self, conn: &mut Connection) -> Result<()> {
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

fn migrations() -> &'static MigrationsConfig {
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

#[derive(Debug)]
pub struct SqliteRegistry {
    conn: Connection,
}

fn file_type_to_sql(file: FilePath) -> (i32, &Path) {
    match file {
        FilePath::Symlink(p) => (1, p),
        FilePath::Directory(p) => (2, p),
    }
}

fn file_type_from_sql(file_type: i32, path: PathBuf) -> Result<FilePathBuf> {
    match file_type {
        1 => Ok(FilePathBuf::Symlink(path)),
        2 => Ok(FilePathBuf::Directory(path)),
        _ => Err(anyhow!("unknown FileType {file_type}")),
    }
}

fn path_to_sql(path: &Path) -> Vec<u8> {
    crate::os_str::to_vec(path.as_os_str().to_os_string())
}

fn path_from_sql(path: Vec<u8>) -> Result<PathBuf> {
    Ok(crate::os_str::from_vec(path)
        .context("failed to parse path")?
        .into())
}

#[derive(Clone, Copy, Debug)]
enum FilePurpose {
    User = 1,
    State = 2,
}

impl SqliteRegistry {
    /// For tests only.
    #[allow(dead_code)]
    fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().context("failed to open in memory")?;
        Self::with_connection(conn)
    }

    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path).with_context(|| format!("failed to open {path:?}"))?;
        Self::with_connection(conn).with_context(|| format!("failed to initialise {path:?}"))
    }

    fn query_app_id(conn: &Connection) -> Result<i32> {
        conn.query_row("PRAGMA application_id", [], |row| row.get(0))
            .context("PRAGMA application_id")
    }

    fn init_app_id(conn: &Connection) -> Result<()> {
        let mut app_id: i32 = Self::query_app_id(conn)?;
        if app_id == 0 {
            // Not initialised.
            conn.pragma_update(None, "application_id", APPLICATION_ID)?;
            app_id = Self::query_app_id(conn)?;
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
        Ok(())
    }

    fn with_connection(mut conn: Connection) -> Result<Self> {
        Self::init_app_id(&conn)?;
        Self::configure_connection(&conn)?;
        migrations()
            .to_stable(&mut conn)
            .context("failed to migrate")?;
        Ok(Self { conn })
    }

    pub fn close(self) -> Result<(), (SqliteRegistry, Error)> {
        match self.conn.close() {
            Ok(()) => Ok(()),
            Err((conn, err)) => Err((
                Self { conn },
                Error::new(err).context("failed to close connection"),
            )),
        }
    }

    fn register_file(&mut self, purpose: FilePurpose, file: FilePath) -> Result<()> {
        let (sql_type, path) = file_type_to_sql(file);
        let mut stmt = self
            .conn
            .prepare_cached(
                "
                INSERT INTO files
                (purpose, file_type, path)
                VALUES (:purpose, :file_type, :path)
                ",
            )
            .context("failed to prepare statement")?;
        stmt.execute(named_params![
            ":purpose": purpose as isize,
            ":file_type": sql_type,
            ":path": path_to_sql(path),
        ])
        .with_context(|| format!("failed to register {path:?}"))?;
        Ok(())
    }

    fn files(&self, purpose: FilePurpose) -> Result<Vec<FilePathBuf>> {
        let mut stmt = self
            .conn
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
            .query_map(named_params![":purpose": purpose as isize], |row| {
                let file_type: i32 = row.get(0)?;
                let path = path_from_sql(row.get(1)?).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(1, Type::Blob, e.into())
                })?;
                let file = file_type_from_sql(file_type, path).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(0, Type::Integer, e.into())
                })?;
                Ok(file)
            })
            .context("failed to query files")?
            .collect();
        files.context("query files")
    }

    fn clear_files(&self, purpose: FilePurpose) -> Result<()> {
        self.conn
            .execute(
                "
                DELETE FROM files
                WHERE
                    purpose = :purpose
                ",
                named_params![":purpose": purpose as isize],
            )
            .with_context(|| format!("clear {purpose:?} files"))?;
        Ok(())
    }
}

impl Registry for SqliteRegistry {
    fn register_user_file(&mut self, file: FilePath) -> Result<()> {
        self.register_file(FilePurpose::User, file)
    }
    fn user_files(&self) -> Result<Vec<FilePathBuf>> {
        self.files(FilePurpose::User)
    }
    fn clear_user_files(&mut self) -> Result<()> {
        self.clear_files(FilePurpose::User)
    }
    fn register_state_file(&mut self, file: FilePath) -> Result<()> {
        self.register_file(FilePurpose::State, file)
    }
    fn state_files(&self) -> Result<Vec<FilePathBuf>> {
        self.files(FilePurpose::State)
    }
    fn clear_state_files(&mut self) -> Result<()> {
        self.clear_files(FilePurpose::State)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use rstest_reuse::{apply, template};

    use super::*;

    #[template]
    #[rstest]
    #[case(FilePurpose::User, FilePurpose::State)]
    #[case(FilePurpose::State, FilePurpose::User)]
    fn sqlite_registry_test(#[case] purpose: FilePurpose, #[case] other_purpose: FilePurpose) {}

    #[apply(sqlite_registry_test)]
    fn test_register_file(purpose: FilePurpose, other_purpose: FilePurpose) {
        let mut reg = SqliteRegistry::open_in_memory().expect("open_in_memory");

        reg.register_file(purpose, FilePath::new_symlink("/test/file"))
            .expect("register_file");

        assert_eq!(
            reg.files(purpose).unwrap(),
            vec![FilePathBuf::new_symlink("/test/file")]
        );
        assert_eq!(reg.files(other_purpose).unwrap(), Vec::<FilePathBuf>::new());
    }

    #[apply(sqlite_registry_test)]
    fn test_clear_files(purpose: FilePurpose, other_purpose: FilePurpose) {
        let mut reg = SqliteRegistry::open_in_memory().expect("open_in_memory");
        reg.register_file(purpose, FilePath::new_symlink("/test/file"))
            .expect("register_file");
        assert_eq!(
            reg.files(purpose).unwrap(),
            vec![FilePathBuf::new_symlink("/test/file")]
        );

        reg.clear_files(purpose).expect("clear_files");

        assert_eq!(reg.files(purpose).unwrap(), Vec::<FilePathBuf>::new());
        assert_eq!(reg.files(other_purpose).unwrap(), Vec::<FilePathBuf>::new());
    }

    #[apply(sqlite_registry_test)]
    fn test_clear_files_does_not_delete_other_files(
        purpose: FilePurpose,
        other_purpose: FilePurpose,
    ) {
        let mut reg = SqliteRegistry::open_in_memory().expect("open_in_memory");
        reg.register_file(purpose, FilePath::new_symlink("/test/file"))
            .expect("register_file");
        assert_eq!(
            reg.files(purpose).unwrap(),
            vec![FilePathBuf::new_symlink("/test/file")]
        );
        reg.register_file(other_purpose, FilePath::new_symlink("/other/file"))
            .expect("register_file");
        assert_eq!(
            reg.files(other_purpose).unwrap(),
            vec![FilePathBuf::new_symlink("/other/file")]
        );

        reg.clear_files(purpose).expect("clear_files");

        assert_eq!(reg.files(purpose).unwrap(), Vec::<FilePathBuf>::new());
        assert_eq!(
            reg.files(other_purpose).unwrap(),
            vec![FilePathBuf::new_symlink("/other/file")]
        );
    }

    #[apply(sqlite_registry_test)]
    fn test_files_order(purpose: FilePurpose, _other_purpose: FilePurpose) {
        let mut reg = SqliteRegistry::open_in_memory().expect("open_in_memory");
        let files = vec![
            FilePath::new_symlink("/test/2/file/1"),
            FilePath::new_symlink("/test/1/file/2"),
            FilePath::new_symlink("/test/3/file/3"),
        ];

        for f in files.iter().copied() {
            reg.register_file(purpose, f).expect("register_file");
        }

        assert_eq!(files, reg.files(purpose).unwrap());
    }

    #[test]
    fn test_application_id_create() {
        let reg = SqliteRegistry::open_in_memory().expect("open_in_memory");

        let id: i32 = reg
            .conn
            .query_row("PRAGMA application_id", [], |row| row.get(0))
            .unwrap();

        assert_eq!(id, APPLICATION_ID);
    }

    #[test]
    fn test_application_id_matching() {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "application_id", APPLICATION_ID)
            .unwrap();

        let reg = SqliteRegistry::with_connection(conn).unwrap();

        let id: i32 = reg
            .conn
            .query_row("PRAGMA application_id", [], |row| row.get(0))
            .unwrap();
        assert_eq!(id, APPLICATION_ID);
    }

    #[test]
    fn test_application_id_not_matching() {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "application_id", 123).unwrap();

        let err = SqliteRegistry::with_connection(conn).unwrap_err();

        assert_eq!(
            err.to_string(),
            "unexpected application_id 7b, want 12fe0c02"
        );
    }

    #[test]
    fn test_migrations_empty_to_stable() {
        let mut conn = Connection::open_in_memory().unwrap();

        migrations().to_stable(&mut conn).expect("must be ok");
    }

    #[test]
    fn test_migrations_empty_to_unstable() {
        let mut conn = Connection::open_in_memory().unwrap();

        migrations().to_unstable(&mut conn).expect("must be ok");
    }

    #[test]
    fn test_migrations_stable_to_unstable() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrations().to_stable(&mut conn).expect("must be ok");

        migrations().to_unstable(&mut conn).expect("must be ok");
    }

    #[test]
    fn test_migrations_unstable_to_stable() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrations().to_unstable(&mut conn).expect("must be ok");

        migrations().to_stable(&mut conn).expect("must be ok");
    }

    #[test]
    fn test_migrations_database_too_far_ahead() {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "user_version", 1000).unwrap();

        let err = SqliteRegistry::with_connection(conn).unwrap_err();

        assert_eq!(err.to_string(), "failed to migrate");
        let err = err.downcast::<rusqlite_migration::Error>().unwrap();
        assert_eq!(
            err,
            rusqlite_migration::Error::MigrationDefinition(
                rusqlite_migration::MigrationDefinitionError::DatabaseTooFarAhead
            )
        );
    }
}
