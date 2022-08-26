use std::path::{Path, PathBuf};

use crate::repository::Repository;

use anyhow::{Context, Result};

const APPS: &str = "apps";
const OVERLAY: &str = "overlay.d";

fn repositories_dirs(root: &Path) -> Result<Vec<PathBuf>> {
    let apps = root.join(APPS);
    let overlays = root.join(OVERLAY);
    let mut result = Vec::<PathBuf>::new();
    result.push(apps);
    let dirs = overlays
        .read_dir()
        .with_context(|| format!("failed to read {}", overlays.display()))?;
    for entry in dirs {
        let dir = entry?;
        let md = std::fs::metadata(dir.path())
            .with_context(|| format!("failed to read metadata for {}", dir.path().display()))?;
        if !md.is_dir() {
            continue;
        }
        result.push(dir.path());
    }
    return Ok(result);
}

pub fn repositories(root: &Path) -> Result<Vec<Repository>> {
    let mut result = Vec::<Repository>::new();
    for dir in repositories_dirs(root)? {
        result.push(Repository::new(dir)?);
    }
    return Ok(result);
}

fn update_repository(root: &Path) -> Result<()> {
    println!("{} $ git pull", root.display());
    return Ok(());
}

pub fn update(root: &Path) -> Result<()> {
    for dir in repositories_dirs(root)? {
        update_repository(&dir)?;
    }
    return Ok(());
}
