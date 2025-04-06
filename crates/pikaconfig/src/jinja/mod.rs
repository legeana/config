mod context;
mod glob;
mod helpers;

use std::ffi::{OsStr, OsString};

use minijinja::{Error as JError, ErrorKind as JErrorKind};
use thiserror::Error as ThisError;

pub(crate) use context::Context;
pub(crate) use helpers::register;

// https://github.com/dtolnay/anyhow/issues/153#issuecomment-833718851
#[derive(Debug, ThisError)]
pub(crate) enum Error {
    #[error("failed to convert {0} {1:?} to string")]
    OsStrToString(&'static str, OsString),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;
pub(crate) type JResult<T, E = JError> = std::result::Result<T, E>;

impl From<Error> for JError {
    fn from(value: Error) -> Self {
        Self::from(JErrorKind::InvalidOperation).with_source(value)
    }
}

pub(crate) fn map_error(e: Error) -> JError {
    JError::from(e)
}

pub(crate) fn map_anyhow(e: impl Into<anyhow::Error>) -> JError {
    Error::Anyhow(e.into()).into()
}

pub(crate) fn to_string(name: &'static str, b: impl AsRef<OsStr>) -> Result<String> {
    let b = b.as_ref();
    b.to_str()
        .map(String::from)
        .ok_or_else(|| Error::OsStrToString(name, b.to_owned()))
}
