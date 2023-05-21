use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use base64::engine::general_purpose::URL_SAFE;
use base64::Engine;
use sha2::{Digest, Sha256};

use super::file_util;
use crate::{package::Module, registry::Registry};

type StateType = &'static str;
const FILE_STATE: StateType = "output";
const DIR_STATE: StateType = "dirs";

fn path_hash(path: &Path) -> Result<PathBuf> {
    let path_str = path
        .to_str()
        .ok_or_else(|| anyhow!("unable to convert {path:?} path to string"))?;

    let mut hasher = Sha256::new();
    hasher.update(path_str.as_bytes());
    let result = hasher.finalize();

    // URL_SAFE is used for compatibility with Python version of pikaconfig.
    Ok(URL_SAFE.encode(result).into())
}

fn state_path(path: &Path, state_type: StateType) -> Result<PathBuf> {
    let hash = path_hash(path).with_context(|| format!("unable to make hash of {path:?}"))?;
    // TODO: Windows/MacOS
    let output_state = dirs::state_dir()
        .ok_or_else(|| anyhow!("failed to get state dir"))?
        .join("pikaconfig")
        .join(state_type);
    Ok(output_state.join(hash))
}

pub struct FileState {
    state: PathBuf,
    dst: PathBuf,
}

impl FileState {
    pub fn new(dst: PathBuf) -> Result<Self> {
        let state = state_path(&dst, FILE_STATE)
            .with_context(|| format!("failed to build state path for {dst:?}"))?;
        Ok(Self { state, dst })
    }
    pub fn path(&self) -> &Path {
        &self.state
    }
    pub fn link(&self) -> &Path {
        &self.dst
    }
}

impl Module for FileState {
    fn install(&self, registry: &mut dyn Registry) -> Result<()> {
        let state_dir = self
            .state
            .parent()
            .ok_or_else(|| anyhow!("failed to get {:?} parent", self.state))?;
        std::fs::create_dir_all(state_dir)
            .with_context(|| format!("failed to create {state_dir:?}"))?;
        file_util::make_symlink(registry, &self.state, &self.dst)
    }
}

pub struct DirectoryState {
    state: PathBuf,
    dst: PathBuf,
}

impl DirectoryState {
    pub fn new(dst: PathBuf) -> Result<Self> {
        let state = state_path(&dst, DIR_STATE)
            .with_context(|| format!("failed to build state path for {dst:?}"))?;
        Ok(Self { state, dst })
    }
    pub fn path(&self) -> &Path {
        &self.state
    }
    pub fn link(&self) -> &Path {
        &self.dst
    }
}

impl Module for DirectoryState {
    fn install(&self, registry: &mut dyn Registry) -> Result<()> {
        std::fs::create_dir_all(&self.state)
            .with_context(|| format!("failed to create {:?}", &self.state))?;
        file_util::make_symlink(registry, &self.state, &self.dst)
    }
}
