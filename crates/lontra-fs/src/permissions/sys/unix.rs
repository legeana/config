use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;

use crate::permissions::Permissions;

pub(in crate::permissions) struct SysPermissions;

impl Permissions for SysPermissions {
    fn set_file_executable(f: &fs::File) -> anyhow::Result<()> {
        let metadata = f.metadata()?;
        let mut perm = metadata.permissions();
        perm.set_mode(perm.mode() | 0o111);
        f.set_permissions(perm)?;
        Ok(())
    }
    fn set_path_executable(path: &Path) -> anyhow::Result<()> {
        let metadata = path.metadata()?;
        let mut perm = metadata.permissions();
        perm.set_mode(perm.mode() | 0o111);
        fs::set_permissions(path, perm)?;
        Ok(())
    }
}
