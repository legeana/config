use std::fs;
use std::path::Path;

use anyhow::{Context as _, Result, anyhow};
use lontra_fs::errkind;
use lontra_fs::symlinks;
use lontra_registry::{FilePath, Registry};

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

pub(super) fn make_symlink(registry: &mut dyn Registry, src: &Path, dst: &Path) -> Result<()> {
    if let Err(err) = errkind::skip_not_found(symlinks::remove(dst)) {
        return Err(err.context(format!("unable to overwrite {dst:?} by {src:?}")));
    }
    let parent = dst
        .parent()
        .ok_or_else(|| anyhow!("unable to get parent of {dst:?}"))?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {parent:?}"))?;
    symlink(src, dst).with_context(|| format!("failed to create a symlink {src:?} -> {dst:?}"))?;
    registry
        .register_user_file(FilePath::Symlink(dst))
        .with_context(|| format!("failed to register symlink {dst:?}"))?;
    log::info!("Symlink {src:?} -> {dst:?}");
    Ok(())
}

#[cfg(unix)]
pub(super) fn set_file_executable(f: &fs::File) -> Result<()> {
    use std::os::unix::fs::PermissionsExt as _;
    let metadata = f.metadata()?;
    let mut perm = metadata.permissions();
    perm.set_mode(perm.mode() | 0o111);
    f.set_permissions(perm)?;
    Ok(())
}

#[cfg(unix)]
pub(super) fn set_path_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt as _;
    let metadata = path.metadata()?;
    let mut perm = metadata.permissions();
    perm.set_mode(perm.mode() | 0o111);
    fs::set_permissions(path, perm)?;
    Ok(())
}

#[cfg(windows)]
pub(super) fn set_file_executable(_f: &fs::File) -> Result<()> {
    // Nothing to do on Windows.
    Ok(())
}

#[cfg(windows)]
pub(super) fn set_path_executable(_path: &Path) -> Result<()> {
    // Nothing to do on Windows.
    Ok(())
}
