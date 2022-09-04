use std::fs;
use std::os::unix;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

pub fn make_symlink(src: &Path, dst: &Path) -> Result<()> {
    if dst.exists() {
        if !dst.is_symlink() {
            return Err(anyhow!(
                "unable to overwrite {} by {}: destination is not a symlink",
                dst.display(),
                src.display(),
            ));
        }
        fs::remove_file(dst).with_context(|| format!("failed to remove {}", dst.display()))?;
    }
    let parent = dst
        .parent()
        .ok_or(anyhow!("unable to get parent of {}", dst.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    unix::fs::symlink(src, dst).with_context(|| {
        format!(
            "failed to create a symlink {} -> {}",
            src.display(),
            dst.display(),
        )
    })?;
    Ok(())
}

pub fn make_local_state(dst: &Path) -> Result<PathBuf> {
    let state = super::local_state::make_state(dst)
        .with_context(|| format!("unable to make local state for {}", dst.display()))?;
    make_symlink(&state, dst)?;
    return Ok(state);
}
