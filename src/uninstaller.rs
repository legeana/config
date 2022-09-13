use std::path::Path;

use anyhow::{anyhow, Context, Result};

use crate::registry::Registry;

pub trait Uninstaller {
    fn uninstall(&mut self) -> Result<()>;
}

impl<T> Uninstaller for T
where
    T: Registry,
{
    fn uninstall(&mut self) -> Result<()> {
        let paths = self
            .paths()
            .with_context(|| "failed to get installed files")?;
        for path in paths.iter().rev() {
            if let Err(err) = remove(path) {
                log::error!("Failed to remove {path:?}: {err}");
            }
        }
        self.clear()
    }
}

fn remove_symlink(path: &Path) -> Result<()> {
    if let Err(err) = std::fs::remove_file(path) {
        if err.kind() == std::io::ErrorKind::NotFound {
            log::debug!("Nothing to remove: {path:?}");
            return Ok(());
        }
        return Err(err).with_context(|| format!("failed to remove {path:?}"));
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
