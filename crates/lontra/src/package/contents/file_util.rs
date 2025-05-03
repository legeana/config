use std::fs;
use std::path::Path;

use anyhow::{Context as _, Result, anyhow};
use lontra_fs::errkind;
use lontra_fs::symlinks;
use lontra_registry::{FilePath, Registry};

pub(super) fn make_symlink(registry: &mut dyn Registry, src: &Path, dst: &Path) -> Result<()> {
    if let Err(err) = errkind::skip_not_found(symlinks::remove(dst)) {
        return Err(err.context(format!("unable to overwrite {dst:?} by {src:?}")));
    }
    let parent = dst
        .parent()
        .ok_or_else(|| anyhow!("unable to get parent of {dst:?}"))?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {parent:?}"))?;
    symlinks::symlink(src, dst)
        .with_context(|| format!("failed to create a symlink {src:?} -> {dst:?}"))?;
    registry
        .register_user_file(FilePath::Symlink(dst))
        .with_context(|| format!("failed to register symlink {dst:?}"))?;
    log::info!("Symlink {src:?} -> {dst:?}");
    Ok(())
}
