mod sys;

use std::fs;
use std::path::Path;

use anyhow::Result;

trait Permissions {
    fn set_file_executable(f: &fs::File) -> Result<()>;
    fn set_path_executable(path: &Path) -> Result<()>;
}

pub fn set_file_executable(f: &fs::File) -> Result<()> {
    sys::SysPermissions::set_file_executable(f)
}

pub fn set_path_executable(path: &Path) -> Result<()> {
    sys::SysPermissions::set_path_executable(path)
}
