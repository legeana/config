use std::fs;
use std::path::Path;

use crate::permissions::Permissions;

pub(in crate::permissions) struct SysPermissions;

impl Permissions for SysPermissions {
    fn set_file_executable(_f: &fs::File) -> anyhow::Result<()> {
        // Nothing to do on Windows.
        Ok(())
    }
    fn set_path_executable(_path: &Path) -> anyhow::Result<()> {
        // Nothing to do on Windows.
        Ok(())
    }
}
