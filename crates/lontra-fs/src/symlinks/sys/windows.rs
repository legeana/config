use std::fs;
use std::os::windows;
use std::os::windows::fs::FileTypeExt as _;
use std::path::Path;

use anyhow::Context as _;
use anyhow::Result;
use anyhow::bail;

use crate::symlinks::Symlinker;

pub(in crate::symlinks) struct SysSymlinker;

impl Symlinker for SysSymlinker {
    fn remove(path: &Path) -> Result<()> {
        let md: fs::Metadata = path
            .symlink_metadata()
            .with_context(|| format!("failed to get {path:?} metadata"))?;
        if md.file_type().is_symlink_file() {
            fs::remove_file(path)?;
            Ok(())
        } else if md.file_type().is_symlink_dir() {
            fs::remove_dir(path)?;
            Ok(())
        } else {
            bail!("{path:?} is not a symlink");
        }
    }
    fn symlink(src: &Path, dst: &Path) -> Result<()> {
        if src.is_dir() {
            windows::fs::symlink_dir(src, dst)?;
        } else {
            windows::fs::symlink_file(src, dst)?;
        }
        Ok(())
    }
}
