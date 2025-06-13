use sqlx::migrate::Migrator;

pub(crate) fn config() -> &'static Migrator {
    static MIGRATOR: Migrator = sqlx::migrate!();
    &MIGRATOR
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use sqlx::Connection as _;
    use sqlx::Row as _;
    use sqlx::SqliteConnection;
    use sqlx::sqlite::SqliteConnectOptions;
    use test_case::test_case;

    use super::*;

    #[test_case("files")]
    #[test_case("updates")]
    fn test_migrations_table_exists(table: &str) {
        crate::runtime::block_on(async {
            let opt = SqliteConnectOptions::new().in_memory(true);
            let mut conn = SqliteConnection::connect_with(&opt)
                .await
                .expect("open_in_memory");

            config().run(&mut conn).await.expect("migrate");

            // Checks table presence.
            let count: i32 = sqlx::query(&format!("SELECT COUNT(*) AS count FROM {table}"))
                .fetch_one(&mut conn)
                .await
                .expect("fetch count")
                .get("count");
            assert_eq!(count, 0);
        });
    }
}
