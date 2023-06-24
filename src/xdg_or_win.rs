#[cfg(unix)]
mod platform {
    use std::path::PathBuf;

    use crate::xdg;

    pub fn cache_dir() -> Option<PathBuf> {
        xdg::cache_dir()
    }

    pub fn config_dir() -> Option<PathBuf> {
        xdg::config_dir()
    }

    pub fn config_local_dir() -> Option<PathBuf> {
        xdg::config_dir()
    }

    pub fn data_dir() -> Option<PathBuf> {
        xdg::data_dir()
    }

    pub fn data_local_dir() -> Option<PathBuf> {
        xdg::data_dir()
    }
}

#[cfg(windows)]
mod platform {
    use std::path::PathBuf;

    pub fn cache_dir() -> Option<PathBuf> {
        dirs::cache_dir()
    }

    pub fn config_dir() -> Option<PathBuf> {
        dirs::config_dir()
    }

    pub fn config_local_dir() -> Option<PathBuf> {
        dirs::config_local_dir()
    }

    pub fn data_dir() -> Option<PathBuf> {
        dirs::data_dir()
    }

    pub fn data_local_dir() -> Option<PathBuf> {
        dirs::data_local_dir()
    }
}

// Explicit use statements to make debugging divergence easier.
pub use platform::cache_dir;
pub use platform::config_dir;
pub use platform::config_local_dir;
pub use platform::data_dir;
pub use platform::data_local_dir;
