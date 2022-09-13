use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use sha2::{Digest, Sha256};

fn path_hash(path: &Path) -> Result<PathBuf> {
    let path_str = path
        .to_str()
        .ok_or_else(|| anyhow!("unable to convert {path:?} path to string"))?;

    let mut hasher = Sha256::new();
    hasher.update(path_str.as_bytes());
    let result = hasher.finalize();

    // URL_SAFE is used for compatibility with Python version of pikaconfig.
    Ok(base64::encode_config(result, base64::URL_SAFE).into())
}

pub fn state_path(path: &Path) -> Result<PathBuf> {
    let hash = path_hash(path).with_context(|| format!("unable to make hash of {path:?}"))?;
    // TODO: Windows/MacOS
    let output_state = dirs::state_dir()
        .ok_or_else(|| anyhow!("failed to get state dir"))?
        .join("pikaconfig")
        .join("output");
    Ok(output_state.join(hash))
}

pub fn make_state(path: &Path) -> Result<PathBuf> {
    let state_path =
        state_path(path).with_context(|| format!("failed to generate state_path for {path:?}"))?;
    let state_dir = state_path
        .parent()
        .ok_or_else(|| anyhow!("failed to get {state_path:?} parent"))?;
    std::fs::create_dir_all(state_dir)
        .with_context(|| format!("failed to create {state_dir:?}"))?;
    Ok(state_path)
}
