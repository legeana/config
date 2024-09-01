use std::path::{Path, PathBuf};

use anyhow::Context;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, Value, ValueRef};

use super::file_type;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct UpdateId(pub(super) Option<i64>);

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum FilePurpose {
    User = 1,
    State = 2,
}

impl ToSql for FilePurpose {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(Value::Integer(*self as i64)))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SqlPath<'a>(pub &'a Path);

impl<'a> ToSql for SqlPath<'a> {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let encoded = crate::os_str::to_vec(self.0.as_os_str().to_os_string());
        Ok(ToSqlOutput::Owned(Value::Blob(encoded)))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SqlPathBuf(pub PathBuf);

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

impl ToSql for file_type::Type {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let value: i64 = match self {
            Self::Symlink(()) => 1,
            Self::Directory(()) => 2,
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
            1 => Ok(file_type::Type::Symlink(())),
            2 => Ok(file_type::Type::Directory(())),
            unknown => Err(FromSqlError::OutOfRange(unknown)),
        }
    }
}
