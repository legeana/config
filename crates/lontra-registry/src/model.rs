use std::path::{Path, PathBuf};

use anyhow::Context as _;
use anyhow::Result;
use lontra_strings::os_str;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, Value, ValueRef};

use crate::proxied;

use super::file_type;

#[derive(Clone, Copy, Debug, PartialEq, sqlx::Type)]
#[sqlx(transparent)]
pub(crate) struct UpdateId(pub(crate) Option<i64>);

impl ToSql for UpdateId {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        match self.0 {
            Some(v) => Ok(ToSqlOutput::Owned(Value::Integer(v))),
            None => Ok(ToSqlOutput::Owned(Value::Null)),
        }
    }
}

impl FromSql for UpdateId {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let Ok(value) = value.as_i64_or_null() else {
            return Err(FromSqlError::InvalidType);
        };
        Ok(Self(value))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, sqlx::Type)]
#[repr(i32)]
pub(crate) enum FilePurpose {
    User = 1,
    State = 2,
}

impl ToSql for FilePurpose {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(Value::Integer(*self as i64)))
    }
}

impl FromSql for FilePurpose {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let Ok(value) = value.as_i64() else {
            return Err(FromSqlError::InvalidType);
        };
        match value {
            v if v == Self::User as i64 => Ok(Self::User),
            v if v == Self::State as i64 => Ok(Self::State),
            unknown => Err(FromSqlError::OutOfRange(unknown)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SqlPath<'a>(pub &'a Path);

impl proxied::Type for SqlPath<'_> {
    type Proxy = Vec<u8>;

    fn to_proxy(&self) -> Result<Self::Proxy> {
        Ok(os_str::to_vec(self.0.into()))
    }
    fn into_proxy(self) -> Result<Self::Proxy> {
        Ok(os_str::to_vec(self.0.into()))
    }
}

crate::sqlx_unsized_impl!(SqlPath<'_>);

impl ToSql for SqlPath<'_> {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let encoded = os_str::to_vec(self.0.as_os_str().to_os_string());
        Ok(ToSqlOutput::Owned(Value::Blob(encoded)))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SqlPathBuf(pub PathBuf);

impl proxied::Type for SqlPathBuf {
    type Proxy = Vec<u8>;

    fn to_proxy(&self) -> Result<Self::Proxy> {
        Ok(os_str::to_vec(self.0.as_os_str().to_owned()))
    }
    fn into_proxy(self) -> Result<Self::Proxy> {
        Ok(os_str::to_vec(self.0.into_os_string()))
    }
}

impl proxied::SizedType for SqlPathBuf {
    fn from_proxy(proxy: Self::Proxy) -> Result<Self> {
        Ok(Self(os_str::from_vec(proxy)?.into()))
    }
}

crate::sqlx_impl!(SqlPathBuf);

impl ToSql for SqlPathBuf {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let encoded = os_str::to_vec(self.0.as_os_str().to_os_string());
        Ok(ToSqlOutput::Owned(Value::Blob(encoded)))
    }
}

impl FromSql for SqlPathBuf {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let Ok(blob) = value.as_blob() else {
            return Err(FromSqlError::InvalidType);
        };
        let decoded = os_str::from_vec(blob.to_vec())
            .context("failed to parse path")
            .map_err(|e| FromSqlError::Other(e.into()))?;
        Ok(Self(decoded.into()))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, sqlx::Type)]
#[repr(i32)]
pub(crate) enum SqlFileType {
    Symlink = 1,
    Directory = 2,
}

impl proxied::Type for file_type::Type {
    type Proxy = SqlFileType;

    fn to_proxy(&self) -> Result<Self::Proxy> {
        let r = match self {
            Self::Symlink => SqlFileType::Symlink,
            Self::Directory => SqlFileType::Directory,
        };
        Ok(r)
    }
    fn into_proxy(self) -> Result<Self::Proxy> {
        self.to_proxy()
    }
}

impl proxied::SizedType for file_type::Type {
    fn from_proxy(proxy: Self::Proxy) -> Result<Self> {
        let r = match proxy {
            Self::Proxy::Symlink => Self::Symlink,
            Self::Proxy::Directory => Self::Directory,
        };
        Ok(r)
    }
}

crate::sqlx_impl!(file_type::Type);

impl ToSql for file_type::Type {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let value: i64 = match self {
            Self::Symlink => 1,
            Self::Directory => 2,
        };
        Ok(ToSqlOutput::Owned(Value::Integer(value)))
    }
}

impl FromSql for file_type::Type {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let Ok(value) = value.as_i64() else {
            return Err(FromSqlError::InvalidType);
        };
        match value {
            1 => Ok(Self::Symlink),
            2 => Ok(Self::Directory),
            unknown => Err(FromSqlError::OutOfRange(unknown)),
        }
    }
}

#[cfg(test)]
mod tests {
    use sqlx::Connection as _;
    use sqlx::Row as _;
    use sqlx::sqlite::SqliteConnectOptions;
    use sqlx::sqlite::SqliteConnection;
    use test_case::test_case;

    use super::*;

    async fn open_in_memory() -> SqliteConnection {
        let opts = SqliteConnectOptions::new().in_memory(true);
        SqliteConnection::connect_with(&opts)
            .await
            .expect("open_in_memory")
    }

    macro_rules! query_into {
        ($input:expr) => {
            async {
                let mut conn = open_in_memory().await;
                sqlx::query("SELECT ? AS result")
                    .bind($input)
                    .fetch_one(&mut conn)
                    .await
                    .expect("fetch_one")
                    .get("result")
            }
        };
    }

    #[test_case(UpdateId(None), None)]
    #[test_case(UpdateId(Some(123)), Some(123))]
    fn test_update_id_encode(update_id: UpdateId, want: Option<i64>) {
        crate::runtime::block_on(async {
            let res: Option<i64> = query_into!(update_id).await;
            assert_eq!(res, want);
        });
    }

    #[test_case(None, UpdateId(None))]
    #[test_case(Some(123), UpdateId(Some(123)))]
    fn test_update_id_decode(update_id: Option<i64>, want: UpdateId) {
        crate::runtime::block_on(async {
            let res: UpdateId = query_into!(update_id).await;
            assert_eq!(res, want);
        });
    }

    #[test_case(FilePurpose::User, 1)]
    #[test_case(FilePurpose::State, 2)]
    fn test_file_purpose_encode(file_purpose: FilePurpose, want: i32) {
        crate::runtime::block_on(async {
            let res: i32 = query_into!(file_purpose).await;
            assert_eq!(res, want);
        });
    }

    #[test_case(1, FilePurpose::User)]
    #[test_case(2, FilePurpose::State)]
    fn test_file_purpose_decode(file_purpose: i32, want: FilePurpose) {
        crate::runtime::block_on(async {
            let res: FilePurpose = query_into!(file_purpose).await;
            assert_eq!(res, want);
        });
    }

    #[test_case("")]
    #[test_case("some/path")]
    fn test_sql_path_encode(path: &str) {
        crate::runtime::block_on(async {
            let sql_path: SqlPath<'_> = SqlPath(Path::new(path));
            let want: Vec<u8> = os_str::to_vec(path.into());

            let res: Vec<u8> = query_into!(sql_path).await;

            assert_eq!(res, want);
        });
    }

    #[test_case("")]
    #[test_case("some/path")]
    fn test_sql_path_buf_encode(path: &str) {
        crate::runtime::block_on(async {
            let sql_path: SqlPathBuf = SqlPathBuf(path.into());
            let want: Vec<u8> = os_str::to_vec(path.into());

            let res: Vec<u8> = query_into!(sql_path).await;

            assert_eq!(res, want);
        });
    }

    #[test_case("")]
    #[test_case("some/path")]
    fn test_sql_path_buf_decode(path: &str) {
        crate::runtime::block_on(async {
            let sql_path: Vec<u8> = os_str::to_vec(path.into());
            let want: SqlPathBuf = SqlPathBuf(path.into());

            let res: SqlPathBuf = query_into!(sql_path).await;

            assert_eq!(res, want);
        });
    }

    #[test_case(file_type::Type::Symlink, 1)]
    #[test_case(file_type::Type::Directory, 2)]
    fn test_file_type_encode(file_type: file_type::Type, want: i32) {
        crate::runtime::block_on(async {
            let res: i32 = query_into!(file_type).await;
            assert_eq!(res, want);
        });
    }

    #[test_case(1, file_type::Type::Symlink)]
    #[test_case(2, file_type::Type::Directory)]
    fn test_file_type_decode(file_type: i32, want: file_type::Type) {
        crate::runtime::block_on(async {
            let res: file_type::Type = query_into!(file_type).await;
            assert_eq!(res, want);
        });
    }
}
