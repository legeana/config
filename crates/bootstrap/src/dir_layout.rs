use std::fs::DirEntry;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::git_utils;

const APPS: &str = "apps";
const OVERLAY: &str = "overlay.d";

fn read_dir_sorted(path: &Path) -> Result<Vec<DirEntry>> {
    let mut paths = path
        .read_dir()
        .with_context(|| format!("failed to read directory {path:?}"))?
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("failed to read dir {path:?}"))?;
    paths.sort_by_key(|de| de.file_name());
    Ok(paths)
}

fn overlay_dirs(root: &Path) -> Result<Vec<PathBuf>> {
    let overlays = root.join(OVERLAY);
    let mut result = Vec::<PathBuf>::new();
    for dir in read_dir_sorted(&overlays)
        .with_context(|| format!("failed to read overlays {overlays:?}"))?
    {
        let md = std::fs::metadata(dir.path())
            .with_context(|| format!("failed to read metadata for {:?}", dir.path()))?;
        if !md.is_dir() {
            continue;
        }
        result.push(dir.path());
    }
    Ok(result)
}

pub fn repositories_dirs(root: &Path) -> Result<Vec<PathBuf>> {
    let apps = root.join(APPS);
    let mut result = Vec::<PathBuf>::new();
    result.push(apps);
    result.extend(overlay_dirs(root)?);
    Ok(result)
}

/// Returns true if restart is required.
fn update_repository(root: &Path) -> Result<bool> {
    if root.join(git_utils::GIT_DIR).is_dir() {
        return git_utils::git_pull(root);
    }
    // Unsupported version control system, if any. Skip.
    Ok(false)
}

pub fn update(root: &Path) -> Result<()> {
    update_repository(root)?;
    for overlay in overlay_dirs(root)? {
        update_repository(&overlay)?;
    }
    Ok(())
}
