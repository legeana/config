use std::fs;
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
        if !md.file_type().is_symlink() {
            bail!("{path:?} is not a symlink");
        }
        fs::remove_file(path)?;
        Ok(())
    }
}
