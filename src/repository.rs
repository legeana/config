use std::path::PathBuf;

use crate::package::Package;

use anyhow::{anyhow, Context, Result};

pub struct Repository {
    root: PathBuf,
    name: String,
    packages: Vec<Package>,
}

impl Repository {
    pub fn new(root: PathBuf) -> Result<Self> {
        let name: String = root
            .file_name()
            .ok_or(anyhow!("failed to get {} basename", root.display()))?
            .to_string_lossy()
            .into();
        let mut repository = Repository {
            root,
            name,
            packages: Vec::new(),
        };
        let dirs = repository
            .root
            .read_dir()
            .with_context(|| format!("failed to read {}", repository.root.display()))?;
        for entry in dirs {
            let dir = entry?;
            let md = std::fs::metadata(dir.path())
                .with_context(|| format!("failed to read metadata for {}", dir.path().display()))?;
            if !md.is_dir() {
                continue;
            }
            let package = Package::new(dir.path())
                .with_context(|| format!("failed to load {}", dir.path().display()))?;
            repository.packages.push(package);
        }
        return Ok(repository);
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn list(&self) -> Vec<String> {
        self.packages.iter().map(|p| p.name().to_string()).collect()
    }
}
