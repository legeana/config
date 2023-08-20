use std::path::Path;

use anyhow::{anyhow, Context, Result};

use crate::file_util;
use crate::registry::Registry;
use crate::symlink_util;

pub trait Uninstaller {
    fn uninstall(&mut self) -> Result<()>;
    /// Removes all state.
    fn cleanup(&mut self) -> Result<()>;
}

impl<T> Uninstaller for T
where
    T: Registry,
{
    fn uninstall(&mut self) -> Result<()> {
        let paths = self.user_files().context("failed to get installed files")?;
        remove_all(paths.iter().rev())?;
        self.clear_user_files()
    }
    fn cleanup(&mut self) -> Result<()> {
        let paths = self.state_files().context("failed to get state files")?;
        remove_all(paths.iter().rev())?;
        self.clear_state_files()
    }
}

fn remove_all<P, I>(iter: I) -> Result<()>
where
    P: AsRef<Path>,
    I: Iterator<Item = P>,
{
    for path in iter {
        let path = path.as_ref();
        if let Err(err) = remove(path) {
            log::error!("Failed to remove {path:?}: {err}");
        }
    }
    Ok(())
}

fn remove_symlink(path: &Path) -> Result<()> {
    match file_util::skip_not_found(symlink_util::remove(path)) {
        Ok(Some(_)) => {}
        Ok(None) => {
            log::debug!("Nothing to remove: {path:?}");
            return Ok(());
        }
        Err(err) => {
            return Err(err).with_context(|| format!("failed to remove {path:?}"));
        }
    }
    log::info!("Removed symlink {path:?}");
    match path.parent() {
        Some(parent) => remove_dir(parent),
        None => Ok(()),
    }
}

fn remove_dir(path: &Path) -> Result<()> {
    for dir in path.ancestors() {
        match std::fs::remove_dir(dir) {
            Ok(()) => {
                log::info!("Removed directory {dir:?}");
            }
            Err(_) => {
                // TODO: check if DirectoryNotEmpty when available,
                // see https://github.com/rust-lang/rust/issues/86442
                // If we can't remove this dir, we probably can't remove its parent either.
                break;
            }
        }
    }
    Ok(())
}

fn remove(path: &Path) -> Result<()> {
    let metadata = match path.symlink_metadata() {
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound {
                log::debug!("{path:?} is already removed, skipping");
                return Ok(());
            }
            return Err(err).with_context(|| format!("failed to get {path:?} metadata"));
        }
        Ok(metadata) => metadata,
    };
    if metadata.is_symlink() {
        remove_symlink(path)
    } else if metadata.is_dir() {
        remove_dir(path)
    } else {
        Err(anyhow!("unexpected file type {:?}", metadata.file_type()))
    }
}
