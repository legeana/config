use std::path::Path;

use crate::repository::Repository;

use anyhow::{Context, Result};

const APPS: &str = "apps";
const OVERLAY: &str = "overlay.d";

pub fn repositories(root: &Path) -> Result<Vec<Repository>> {
    let apps = root.join(APPS);
    let overlays = root.join(OVERLAY);
    let mut result = Vec::<Repository>::new();
    result.push(Repository { root: apps });
    let dirs = overlays
        .read_dir()
        .with_context(|| format!("failed to read {}", overlays.display()))?;
    for entry in dirs {
        let dir = entry?;
        result.push(Repository::new(dir.path()));
    }
    Ok(result)
}
