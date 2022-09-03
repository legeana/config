use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use base64;
use dirs;
use sha2::{Digest, Sha256};

fn path_hash(path: &Path) -> Result<PathBuf> {
    let path_str = path.to_str().ok_or(anyhow!(
        "unable to convert {} path to string",
        path.display()
    ))?;

    let mut hasher = Sha256::new();
    hasher.update(path_str.as_bytes());
    let result = hasher.finalize();

    return Ok(base64::encode_config(result, base64::URL_SAFE_NO_PAD).into());
}

pub fn make_state(path: &Path) -> Result<PathBuf> {
    let hash =
        path_hash(path).with_context(|| format!("unable to make hash of {}", path.display()))?;
    // TODO: Windows/MacOS
    let output_state = dirs::state_dir()
        .ok_or(anyhow!("failed to get state dir"))?
        .join("pikaconfig")
        .join("output");
    std::fs::create_dir_all(&output_state)
        .with_context(|| format!("failed to create {}", output_state.display()))?;
    return Ok(output_state.join(hash));
}
