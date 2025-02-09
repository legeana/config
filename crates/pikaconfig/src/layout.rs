use std::ffi::OsStr;
use std::path::Path;

use anyhow::{Context, Result};

use crate::dir_layout;
use crate::repository;
use crate::repository::Repository;

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
        if entry.path().file_name() == Some(OsStr::new(git_utils::GIT_DIR)) {
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
    for repo in dir_layout::repositories_dirs(root)? {
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
