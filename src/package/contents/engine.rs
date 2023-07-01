use std::path::{Path, PathBuf};

pub struct State {
    pub enabled: bool,
    pub prefix: PathBuf,
}

impl State {
    pub fn new() -> Self {
        Self {
            enabled: true,
            prefix: dirs::home_dir().expect("failed to determine home dir"),
        }
    }
    pub fn dst_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.prefix.join(path)
    }
}
