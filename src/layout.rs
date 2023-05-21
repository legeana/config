use std::ffi::OsStr;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};

use crate::git_utils;
use crate::repository;
use crate::repository::Repository;

use anyhow::{Context, Result};

const APPS: &str = "apps";
const OVERLAY: &str = "overlay.d";
const GIT_DIR: &str = ".git";

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

fn repositories_dirs(root: &Path) -> Result<Vec<PathBuf>> {
    let apps = root.join(APPS);
    let mut result = Vec::<PathBuf>::new();
    result.push(apps);
    result.extend(overlay_dirs(root)?);
    Ok(result)
}

fn walk_repositories<F>(root: &Path, mut visit: F) -> Result<()>
where
    F: FnMut(walkdir::DirEntry) -> Result<()>,
{
    let mut it = walkdir::WalkDir::new(root).sort_by_file_name().into_iter();
    while let Some(entry) = it.next() {
        let entry = entry.with_context(|| format!("failed to iterate over {root:?}"))?;
        match entry.metadata() {
            Err(err) => {
                log::warn!(
                    "skipping unknown filesystem entry {:?}: {err}",
                    entry.path()
                );
                continue;
            }
            Ok(md) => {
                if !md.is_dir() {
                    log::debug!("skipping non-directory filesystem entry {:?}", entry.path());
                    continue;
                }
            }
        }
        if entry.path().file_name() == Some(OsStr::new(".git")) {
            it.skip_current_dir();
            continue;
        }
        if !repository::is_repository_dir(entry.path())
            .with_context(|| format!("failed to check if {:?} is a repository", entry.path()))?
        {
            log::debug!("skipping non-repository directory {:?}", entry.path());
            continue;
        }
        log::debug!("found repository at {:?}", entry.path());
        visit(entry)?;
        it.skip_current_dir();
    }

    Ok(())
}

pub fn repositories(root: &Path) -> Result<Vec<Repository>> {
    let mut result = Vec::<Repository>::new();
    for repo in repositories_dirs(root)? {
        walk_repositories(&repo, |dir| {
            log::debug!("loading repository {:?}", dir.path());
            result.push(
                Repository::new(dir.path().to_owned())
                    .with_context(|| format!("failed to load repository {root:?}"))?,
            );
            Ok(())
        })?;
    }
    log::debug!(
        "successfully loaded all repositories in {root:?}: {:?}",
        result.iter().map(|r| r.name()).collect::<Vec<&str>>()
    );
    Ok(result)
}

/// Returns true if restart is required.
fn update_repository(root: &Path) -> Result<bool> {
    if root.join(GIT_DIR).is_dir() {
        return git_utils::git_pull(root);
    }
    // Unsupported version control system, if any. Skip.
    Ok(false)
}

/// Returns true if restart is required.
pub fn update(root: &Path) -> Result<bool> {
    // We restart iff the root repository was updated.
    let updated = update_repository(root)?;
    for overlay in overlay_dirs(root)? {
        update_repository(&overlay)?;
    }
    Ok(updated)
}
