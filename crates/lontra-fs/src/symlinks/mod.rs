mod sys;

use std::path::Path;

use anyhow::Result;

trait Symlinker {
    fn remove(path: &Path) -> Result<()>;
    fn symlink(src: &Path, dst: &Path) -> Result<()>;
}

/// Remove path if it is a symlink.
pub fn remove(path: &Path) -> Result<()> {
    sys::SysSymlinker::remove(path)
}

/// Creates a symlink to `src` in `dst`.
pub fn symlink(src: &Path, dst: &Path) -> Result<()> {
    sys::SysSymlinker::symlink(src, dst)
}
