use std::fs;
use std::os::unix;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use crate::registry::Registry;

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
    unix::fs::symlink(src, dst)
        .with_context(|| format!("failed to create a symlink {src:?} -> {dst:?}"))?;
    registry
        .register(dst)
        .with_context(|| format!("failed to register symlink {dst:?}"))?;
    log::info!("Symlink {src:?} -> {dst:?}");
    Ok(())
}

pub fn make_local_state(registry: &mut dyn Registry, dst: &Path) -> Result<PathBuf> {
    let state = super::local_state::make_state(dst)
        .with_context(|| format!("unable to make local state for {dst:?}"))?;
    make_symlink(registry, &state, dst)?;
    Ok(state)
}
