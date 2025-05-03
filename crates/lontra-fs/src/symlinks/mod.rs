mod sys;

use std::path::Path;

use anyhow::Result;

trait Symlinker {
    fn remove(path: &Path) -> Result<()>;
}

/// Remove path if it is a symlink.
pub fn remove(path: &Path) -> Result<()> {
    sys::SysSymlinker::remove(path)
}
