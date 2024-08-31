use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, Value, ValueRef};

use super::{FilePath, FilePathBuf};

#[derive(Clone, Copy, Debug)]
pub(super) enum FilePurpose {
    User = 1,
    State = 2,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SqlPath<'a>(&'a Path);

impl<'a> ToSql for SqlPath<'a> {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let encoded = crate::os_str::to_vec(self.0.as_os_str().to_os_string());
        Ok(ToSqlOutput::Owned(Value::Blob(encoded)))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SqlPathBuf(PathBuf);

impl ToSql for SqlPathBuf {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let encoded = crate::os_str::to_vec(self.0.as_os_str().to_os_string());
        Ok(ToSqlOutput::Owned(Value::Blob(encoded)))
    }
}

impl FromSql for SqlPathBuf {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let Ok(blob) = value.as_blob() else {
            return Err(FromSqlError::InvalidType);
        };
        let decoded = crate::os_str::from_vec(blob.to_vec())
            .context("failed to parse path")
            .map_err(|e| FromSqlError::Other(e.into()))?;
        Ok(SqlPathBuf(decoded.into()))
    }
}

impl ToSql for FilePurpose {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(Value::Integer(*self as i64)))
    }
}

pub(super) fn file_type_to_sql(file: FilePath) -> (i32, SqlPath) {
    match file {
        FilePath::Symlink(p) => (1, SqlPath(p)),
        FilePath::Directory(p) => (2, SqlPath(p)),
    }
}

pub(super) fn file_type_from_sql(file_type: i32, path: SqlPathBuf) -> Result<FilePathBuf> {
    match file_type {
        1 => Ok(FilePathBuf::Symlink(path.0)),
        2 => Ok(FilePathBuf::Directory(path.0)),
        _ => Err(anyhow!("unknown FileType {file_type}")),
    }
}
