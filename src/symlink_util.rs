use std::fs;
use std::io::Result;
use std::path::Path;

pub struct Metadata {
    metadata: fs::Metadata,
}

impl Metadata {
    #[allow(dead_code)]
    pub fn is_symlink(&self) -> bool {
        self.metadata.is_symlink()
    }
    #[cfg(unix)]
    pub fn is_symlink_file(&self) -> bool {
        self.metadata.is_symlink()
    }
    #[cfg(windows)]
    pub fn is_symlink_file(&self) -> bool {
        use std::os::windows::fs::FileTypeExt;
        self.metadata.file_type().is_symlink_file()
    }
    #[cfg(unix)]
    pub fn is_symlink_dir(&self) -> bool {
        false
    }
    #[cfg(windows)]
    pub fn is_symlink_dir(&self) -> bool {
        use std::os::windows::fs::FileTypeExt;
        self.metadata.file_type().is_symlink_dir()
    }
}

impl Into<fs::Metadata> for Metadata {
    fn into(self) -> fs::Metadata {
        self.metadata
    }
}

impl From<fs::Metadata> for Metadata {
    fn from(metadata: fs::Metadata) -> Self {
        Self { metadata }
    }
}

#[allow(dead_code)]  // This is a good usage example.
pub fn metadata(path: &Path) -> Result<Metadata> {
    let metadata = path.symlink_metadata()?;
    Ok(metadata.into())
}
