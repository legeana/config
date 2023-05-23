use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Context, Result};

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
    let status = Command::new("git")
        .args(["pull", "--ff-only"])
        .current_dir(root)
        .status()
        .with_context(|| format!("{root:?} $ git pull --ff-only"))?;
    if !status.success() {
        return Err(anyhow!("{root:?} $ git pull --ff-only"));
    }
    let new_head = get_head(root)?;
    Ok(old_head != new_head)
}

pub fn git_hard_pull(root: &Path) -> Result<()> {
    let status = Command::new("git")
        .args(["fetch", ORIGIN])
        .current_dir(root)
        .status()
        .with_context(|| format!("{root:?} $ git fetch {ORIGIN}"))?;
    if !status.success() {
        return Err(anyhow!("{root:?} $ git fetch {ORIGIN}"));
    }
    let status = Command::new("git")
        .args(["reset", "--hard"])
        .arg(format!("{ORIGIN}/{HEAD}"))
        .current_dir(root)
        .status()
        .with_context(|| format!("{root:?} $ git reset --hard {ORIGIN}/{HEAD}"))?;
    if !status.success() {
        return Err(anyhow!("{root:?} $ git reset --hard {ORIGIN}/{HEAD}"));
    }
    Ok(())
}

pub fn git_clone(remote: &Remote, root: &Path) -> Result<()> {
    let branch_args = match &remote.branch {
        Some(branch) => vec![format!("--branch={branch}")],
        None => Vec::new(),
    };
    let command = || {
        // TODO: branch
        format!(
            "$ git clone -- {:?} {root:?}",
            remote.url,
        )
    };
    let status = Command::new("git")
        .arg("clone")
        .args(&branch_args)
        .arg("--")
        .arg(&remote.url)
        .arg(root)
        .status()
        .with_context(command)?;
    if !status.success() {
        return Err(anyhow!(command()));
    }
    Ok(())
}

fn get_head(root: &Path) -> Result<String> {
    let rev_parse = Command::new("git")
        .args(["rev-parse", HEAD])
        .current_dir(root)
        .output()
        .with_context(|| format!("{root:?} $ git rev-parse {HEAD}"))?;
    if !rev_parse.status.success() {
        let err = String::from_utf8_lossy(&rev_parse.stdout);
        return Err(anyhow!("failed git rev-parse {HEAD}: {err}"));
    }
    let out = String::from_utf8(rev_parse.stdout.clone()).with_context(|| {
        format!(
            "failed to parse {root:?} $ git rev-parse {HEAD} output {:?}",
            String::from_utf8_lossy(&rev_parse.stdout),
        )
    })?;
    Ok(out.trim().to_string())
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
