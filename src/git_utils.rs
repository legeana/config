use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Context, Result};

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

fn get_head(root: &Path) -> Result<String> {
    let rev_parse = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output()
        .with_context(|| format!("{root:?} $ git rev-parse HEAD"))?;
    if !rev_parse.status.success() {
        let err = String::from_utf8_lossy(&rev_parse.stdout);
        return Err(anyhow!("failed git rev-parse HEAD: {}", err));
    }
    let out = String::from_utf8(rev_parse.stdout.clone()).with_context(|| {
        format!(
            "failed to parse {root:?} $ git rev-parse HEAD output {:?}",
            String::from_utf8_lossy(&rev_parse.stdout),
        )
    })?;
    Ok(out.trim().to_string())
}
