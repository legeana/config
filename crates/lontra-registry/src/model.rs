use std::path::{Path, PathBuf};

use anyhow::Result;
use lontra_strings::os_str;

use crate::proxied;

use super::file_type;

#[derive(Clone, Copy, Debug, PartialEq, sqlx::Type)]
#[sqlx(transparent)]
pub(crate) struct UpdateId(pub(crate) i64);

#[derive(Clone, Copy, Debug, PartialEq, sqlx::Type)]
#[repr(i32)]
pub(crate) enum FilePurpose {
    User = 1,
    State = 2,
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

    #[test_case(None, None)]
    #[test_case(Some(UpdateId(123)), Some(123))]
    fn test_update_id_encode(update_id: Option<UpdateId>, want: Option<i64>) {
        crate::runtime::block_on(async {
            let res: Option<i64> = query_into!(update_id).await;
            assert_eq!(res, want);
        });
    }

    #[test_case(None, None)]
    #[test_case(Some(123), Some(UpdateId(123)))]
    fn test_update_id_decode(update_id: Option<i64>, want: Option<UpdateId>) {
        crate::runtime::block_on(async {
            let res: Option<UpdateId> = query_into!(update_id).await;
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
