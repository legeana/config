use std::path::Path;

use anyhow::{anyhow, Result};
use process_utils::{cmd, Shell};

const ORIGIN: &str = "origin";
const HEAD: &str = "HEAD";
const BRANCH_SEP: char = '#';
pub const GIT_DIR: &str = ".git";

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
    cmd!(["git", "pull", "--ff-only"]).current_dir(root).run()?;
    let new_head = get_head(root)?;
    Ok(old_head != new_head)
}

pub fn git_force_shallow_pull(root: &Path, remote: &Remote) -> Result<()> {
    let mut sh = Shell::new();
    sh.current_dir(root);
    sh.run(cmd!(["git", "remote", "rm", ORIGIN]))?;
    sh.run(cmd!(["git", "remote", "add", ORIGIN, &remote.url]))?;
    sh.run(cmd!(["git", "fetch", "--depth=1", ORIGIN]))?;
    sh.run(cmd!(["git", "remote", "set-head", "--auto", ORIGIN]))?;
    let branch = match &remote.branch {
        Some(branch) => branch.clone(),
        None => get_remote_head_ref(root)?,
    };
    sh.run(cmd!(["git", "checkout", "--force", &branch]))?;
    sh.run(cmd!([
        "git",
        "reset",
        "--hard",
        format!("{ORIGIN}/{branch}")
    ]))?;
    Ok(())
}

pub fn git_shallow_clone(remote: &Remote, root: &Path) -> Result<()> {
    let branch = remote
        .branch
        .as_ref()
        .map(|branch| format!("--branch={branch}"));
    cmd!(
        ["git", "clone", "--depth=1"],
        branch,
        ["--", &remote.url, root],
    )
    .run()
}

fn get_head(root: &Path) -> Result<String> {
    let rev_parse = cmd!(["git", "rev-parse", HEAD])
        .current_dir(root)
        .output()?;
    Ok(rev_parse.trim().to_string())
}

pub fn get_head_ref(root: &Path) -> Result<String> {
    let symbolic_ref = get_symbolic_ref(root, "HEAD")?;
    Ok(symbolic_ref
        .rsplit_once('/')
        .ok_or_else(|| anyhow!("failed to parse {symbolic_ref}"))?
        .1
        .to_string())
}

pub fn get_remote_head_ref(root: &Path) -> Result<String> {
    let symbolic_ref = get_symbolic_ref(root, format!("refs/remotes/{ORIGIN}/{HEAD}"))?;
    Ok(symbolic_ref
        .rsplit_once('/')
        .ok_or_else(|| anyhow!("failed to parse {symbolic_ref}"))?
        .1
        .to_string())
}

fn get_symbolic_ref(root: &Path, name: impl AsRef<str>) -> Result<String> {
    let output = cmd!(["git", "symbolic-ref", "--", name.as_ref()])
        .current_dir(root)
        .output()?;
    Ok(output.trim().to_owned())
}

pub fn get_remote_url(root: &Path) -> Result<String> {
    let url_raw = cmd!(["git", "remote", "get-url", ORIGIN])
        .current_dir(root)
        .output()?;
    Ok(url_raw.trim().to_owned())
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
