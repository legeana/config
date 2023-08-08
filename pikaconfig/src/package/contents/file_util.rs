use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};

use crate::file_util;
use crate::registry::Registry;
use crate::symlink_util;

#[cfg(unix)]
fn symlink(src: &Path, dst: &Path) -> Result<()> {
    use std::os::unix;
    unix::fs::symlink(src, dst)?;
    Ok(())
}

#[cfg(windows)]
fn symlink(src: &Path, dst: &Path) -> Result<()> {
    use std::os::windows::fs;
    if src.is_dir() {
        fs::symlink_dir(src, dst)?;
    } else {
        fs::symlink_file(src, dst)?;
    }
    Ok(())
}

pub fn make_symlink(registry: &mut dyn Registry, src: &Path, dst: &Path) -> Result<()> {
    if let Err(err) = file_util::skip_not_found(symlink_util::remove(dst)) {
        return Err(err.context(format!("unable to overwrite {dst:?} by {src:?}")));
    }
    let parent = dst
        .parent()
        .ok_or_else(|| anyhow!("unable to get parent of {dst:?}"))?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {parent:?}"))?;
    symlink(src, dst).with_context(|| format!("failed to create a symlink {src:?} -> {dst:?}"))?;
    registry
        .register(dst)
        .with_context(|| format!("failed to register symlink {dst:?}"))?;
    log::info!("Symlink {src:?} -> {dst:?}");
    Ok(())
}

#[cfg(unix)]
pub fn set_executable(f: &std::fs::File) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let metadata = f.metadata()?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(permissions.mode() | 0o111);
    f.set_permissions(permissions)?;
    Ok(())
}

#[cfg(windows)]
pub fn set_executable(_f: &std::fs::File) -> Result<()> {
    // Nothing to do on Windows.
    Ok(())
}
