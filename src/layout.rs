use std::path::{Path, PathBuf};

use crate::repository::Repository;

use anyhow::{Context, Result};

const APPS: &str = "apps";
const OVERLAY: &str = "overlay.d";
const GIT_DIR: &str = ".git";

fn overlay_dirs(root: &Path) -> Result<Vec<PathBuf>> {
    let overlays = root.join(OVERLAY);
    let mut result = Vec::<PathBuf>::new();
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

fn repositories_dirs(root: &Path) -> Result<Vec<PathBuf>> {
    let apps = root.join(APPS);
    let mut result = Vec::<PathBuf>::new();
    result.push(apps);
    result.extend(overlay_dirs(root)?);
    return Ok(result);
}

pub fn repositories(root: &Path) -> Result<Vec<Repository>> {
    let mut result = Vec::<Repository>::new();
    for dir in repositories_dirs(root)? {
        result.push(Repository::new(dir)?);
    }
    return Ok(result);
}

fn update_repository(root: &Path) -> Result<bool> {
    if !root.join(GIT_DIR).is_dir() {
        // Skip non-git overlay.
        return Ok(false);
    }
    println!("{} $ git pull", root.display());
    return Ok(true);
}

/// Returns true if restart is required.
pub fn update(root: &Path) -> Result<bool> {
    // We restart iff the root repository was updated.
    let updated = update_repository(root)?;
    for overlay in overlay_dirs(root)? {
        update_repository(&overlay)?;
    }
    return Ok(updated);
}
