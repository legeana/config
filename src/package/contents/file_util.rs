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

fn remove_symlink(path: &Path, md: symlink_util::Metadata) -> Result<()> {
    if md.is_symlink_file() {
        fs::remove_file(path)?;
        Ok(())
    } else if md.is_symlink_dir() {
        fs::remove_dir(path)?;
        Ok(())
    } else {
        Err(anyhow!("{path:?} is not a symlink"))
    }
}

pub fn make_symlink(registry: &mut dyn Registry, src: &Path, dst: &Path) -> Result<()> {
    let md = file_util::skip_not_found(dst.symlink_metadata())
        .with_context(|| format!("failed to read {dst:?} metadata"))?;
    if let Some(md) = md {
        if !md.is_symlink() {
            return Err(anyhow!(
                "unable to overwrite {dst:?} by {src:?}: destination is not a symlink"
            ));
        }
        remove_symlink(dst, md.into()).with_context(|| format!("failed to remove {dst:?}"))?;
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
