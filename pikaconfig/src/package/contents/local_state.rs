use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use base64::engine::general_purpose::URL_SAFE;
use base64::Engine;
use sha2::{Digest, Sha256};

use crate::module::Module;
use crate::registry::Registry;

use super::file_util;

type StateType = &'static str;
const FILE_STATE: StateType = "output";
const DIR_STATE: StateType = "dirs";

fn path_hash(path: &Path) -> Result<PathBuf> {
    let path_str = path
        // TODO: this is not cross-platform.
        // Maybe use to_string_lossy(), os_str_bytes or OsStrExt.
        .to_str()
        .ok_or_else(|| anyhow!("unable to convert {path:?} path to string"))?;

    let mut hasher = Sha256::new();
    hasher.update(path_str.as_bytes());
    let result = hasher.finalize();

    // URL_SAFE is used for compatibility with Python version of pikaconfig.
    Ok(URL_SAFE.encode(result).into())
}

#[cfg(unix)]
fn state_dir() -> Option<PathBuf> {
    dirs::state_dir()
}

#[cfg(windows)]
fn state_dir() -> Option<PathBuf> {
    dirs::data_local_dir()
}

fn state_dir_for_type(state_type: StateType) -> Result<PathBuf> {
    Ok(state_dir()
        .ok_or_else(|| anyhow!("failed to get state dir"))?
        .join("pikaconfig")
        .join(state_type))
}

fn state_path(path: &Path, state_type: StateType) -> Result<PathBuf> {
    let hash = path_hash(path).with_context(|| format!("unable to make hash of {path:?}"))?;
    Ok(state_dir_for_type(state_type)?.join(hash))
}

pub struct StateMapping {
    /// The actual file.
    path: PathBuf,
    /// The destination symlink.
    link: PathBuf,
}

impl StateMapping {
    /// The actual file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl std::fmt::Debug for StateMapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} (actual {:?})", self.link, self.path)
    }
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
    pub fn mapping(&self) -> StateMapping {
        StateMapping {
            path: self.path().to_owned(),
            link: self.link().to_owned(),
        }
    }
    pub fn path(&self) -> &Path {
        &self.state
    }
    pub fn link(&self) -> &Path {
        &self.dst
    }
}

impl Module for FileState {
    fn install(&self, _rules: &super::Rules, registry: &mut dyn Registry) -> Result<()> {
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
    pub fn mapping(&self) -> StateMapping {
        StateMapping {
            path: self.state.clone(),
            link: self.dst.clone(),
        }
    }
}

impl Module for DirectoryState {
    fn install(&self, _rules: &super::Rules, registry: &mut dyn Registry) -> Result<()> {
        std::fs::create_dir_all(&self.state)
            .with_context(|| format!("failed to create {:?}", &self.state))?;
        file_util::make_symlink(registry, &self.state, &self.dst)
    }
}
