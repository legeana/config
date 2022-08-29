use std::path::{Path, PathBuf};
use std::process::Command;

use crate::repository::Repository;

use anyhow::{anyhow, Context, Result};

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

fn get_head(root: &Path) -> Result<String> {
    let rev_parse = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .with_context(|| format!("{} $ git rev-parse HEAD", root.display()))?;
    if !rev_parse.status.success() {
        let err = String::from_utf8_lossy(&rev_parse.stdout);
        return Err(anyhow!("failed git rev-parse HEAD: {}", err));
    }
    let out = String::from_utf8(rev_parse.stdout).with_context(|| {
        format!(
            "failed to parse '{} $ git rev-parse HEAD' output",
            root.display()
        )
    })?;
    return Ok(out.trim().to_string());
}

/// Returns whether pull changed HEAD.
fn git_pull(root: &Path) -> Result<bool> {
    let old_head = get_head(root)?;
    let status = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("pull")
        .arg("--ff-only")
        .status()
        .with_context(|| format!("git -C {} pull --ff-only", root.display()))?;
    if !status.success() {
        return Err(anyhow!("git -C {} pull --ff-only", root.display()));
    }
    let new_head = get_head(root)?;
    return Ok(old_head != new_head);
}

/// Returns true if restart is required.
fn update_repository(root: &Path) -> Result<bool> {
    if root.join(GIT_DIR).is_dir() {
        return git_pull(root);
    }
    // Unsupported version control system, if any. Skip.
    return Ok(false);
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
