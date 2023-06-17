#[cfg(unix)]
mod platform {
    use std::path::PathBuf;

    fn base() -> Option<xdg::BaseDirectories> {
        xdg::BaseDirectories::new().ok()
    }

    pub fn cache_dir() -> Option<PathBuf> {
        base().and_then(|base| Some(base.get_cache_home()))
    }

    pub fn config_dir() -> Option<PathBuf> {
        base().and_then(|base| Some(base.get_config_home()))
    }

    pub fn config_local_dir() -> Option<PathBuf> {
        base().and_then(|base| Some(base.get_config_home()))
    }

    pub fn data_dir() -> Option<PathBuf> {
        base().and_then(|base| Some(base.get_data_home()))
    }

    pub fn data_local_dir() -> Option<PathBuf> {
        base().and_then(|base| Some(base.get_data_home()))
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
