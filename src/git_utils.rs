use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Result};

use crate::process_utils;

const ORIGIN: &str = "origin";
const HEAD: &str = "HEAD";
const BRANCH_SEP: char = '#';

pub struct Remote {
    pub url: String,
    pub branch: Option<String>,
}

impl Remote {
    pub fn new(addr: &str) -> Self {
        match addr.rsplit_once(BRANCH_SEP) {
            Some((url, branch)) => Self {
                url: url.to_owned(),
                branch: Some(branch.to_owned()),
            },
            None => Self {
                url: addr.to_owned(),
                branch: None,
            },
        }
    }
}

/// Returns whether pull changed HEAD.
pub fn git_pull(root: &Path) -> Result<bool> {
    let old_head = get_head(root)?;
    process_utils::run(
        Command::new("git")
            .args(["pull", "--ff-only"])
            .current_dir(root),
    )?;
    let new_head = get_head(root)?;
    Ok(old_head != new_head)
}

pub fn git_hard_pull(root: &Path) -> Result<()> {
    process_utils::run(
        Command::new("git")
            .args(["fetch", ORIGIN])
            .current_dir(root),
    )?;
    process_utils::run(
        Command::new("git")
            .args(["reset", "--hard"])
            .arg(format!("{ORIGIN}/{HEAD}"))
            .current_dir(root),
    )
}

pub fn git_force_remote(root: &Path, remote: &Remote) -> Result<()> {
    process_utils::run(
        Command::new("git")
            .arg("remote")
            .arg("set-url")
            .arg(ORIGIN)
            .arg(&remote.url)
            .current_dir(root),
    )?;
    let branch = match &remote.branch {
        Some(branch) => branch.clone(),
        None => get_remote_head(root)?,
    };
    process_utils::run(
        Command::new("git")
            .arg("checkout")
            .arg("--force")
            .arg(branch)
            .current_dir(root),
    )?;
    Ok(())
}

pub fn git_clone(remote: &Remote, root: &Path) -> Result<()> {
    let mut cmd = Command::new("git");
    cmd.arg("clone");
    if let Some(branch) = &remote.branch {
        cmd.arg(format!("--branch={branch}"));
    }
    cmd.arg("--").arg(&remote.url).arg(root);
    process_utils::run(&mut cmd)
}

fn get_head(root: &Path) -> Result<String> {
    let rev_parse = process_utils::output(
        Command::new("git")
            .args(["rev-parse", HEAD])
            .current_dir(root),
    )?;
    Ok(rev_parse.trim().to_string())
}

fn get_remote_head(root: &Path) -> Result<String> {
    let symbolic_ref = process_utils::output(
        Command::new("git")
            .arg("symbolic-ref")
            .arg(format!("refs/remotes/{ORIGIN}/{HEAD}"))
            .current_dir(root),
    )?;
    Ok(symbolic_ref
        .rsplit_once('/')
        .ok_or_else(|| anyhow!("failed to parse {symbolic_ref}"))?
        .1
        .trim()
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_without_branch() {
        let remote = Remote::new("http://github.com/example/repo.git");
        assert_eq!(remote.url, "http://github.com/example/repo.git");
        assert_eq!(remote.branch, None);
    }

    #[test]
    fn test_remote_with_branch() {
        let remote = Remote::new("http://github.com/example/repo.git#branch");
        assert_eq!(remote.url, "http://github.com/example/repo.git");
        assert_eq!(remote.branch, Some("branch".to_owned()));
    }

    #[test]
    fn test_remote_with_branch_and_hashes() {
        let remote = Remote::new("http://github.com/#example/repo.git#branch");
        assert_eq!(remote.url, "http://github.com/#example/repo.git");
        assert_eq!(remote.branch, Some("branch".to_owned()));
    }
}
