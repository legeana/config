use std::ffi::OsStr;

use anyhow::{Context as _, Result};

pub(crate) fn is_command<T: AsRef<OsStr> + std::fmt::Debug>(cmd: T) -> Result<bool> {
    match which::which(&cmd) {
        Ok(_) => Ok(true),
        Err(which::Error::CannotFindBinaryPath) => Ok(false),
        Err(err) => Err(err).context(format!("failed to check if {cmd:?} is available in PATH")),
    }
}
