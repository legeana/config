use std::path::PathBuf;

pub struct Repository {
    pub root: PathBuf,
}

pub struct Package {
    pub name: String,
}

impl Repository {
    pub fn new(root: PathBuf) -> Self {
        Repository { root }
    }
}
