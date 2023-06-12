use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};

use crate::registry::Registry;

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
    if dst.exists() {
        if !dst.is_symlink() {
            return Err(anyhow!(
                "unable to overwrite {dst:?} by {src:?}: destination is not a symlink"
            ));
        }
        fs::remove_file(dst).with_context(|| format!("failed to remove {dst:?}"))?;
    }
    let parent = dst
        .parent()
        .ok_or_else(|| anyhow!("unable to get parent of {dst:?}"))?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {parent:?}"))?;
    symlink(src, dst)
        .with_context(|| format!("failed to create a symlink {src:?} -> {dst:?}"))?;
    registry
        .register(dst)
        .with_context(|| format!("failed to register symlink {dst:?}"))?;
    log::info!("Symlink {src:?} -> {dst:?}");
    Ok(())
}
