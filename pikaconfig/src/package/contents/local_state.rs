#![allow(dead_code)]

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use base64::engine::general_purpose::URL_SAFE;
use base64::Engine;
use sha2::{Digest, Sha256};

use crate::module::{Module, Rules};
use crate::registry::Registry;

use super::file_util;

#[cfg(unix)]
fn state_dir() -> Option<PathBuf> {
    dirs::state_dir()
}

#[cfg(windows)]
fn state_dir() -> Option<PathBuf> {
    dirs::data_local_dir()
}

trait LocalStateRoot {
    fn path(&self) -> Result<PathBuf>;
}

struct StateType(&'static str);

impl LocalStateRoot for StateType {
    fn path(&self) -> Result<PathBuf> {
        Ok(state_dir()
            .ok_or_else(|| anyhow!("failed to get state dir"))?
            .join("pikaconfig")
            .join(self.0))
    }
}

struct CacheType(&'static str);

impl LocalStateRoot for CacheType {
    fn path(&self) -> Result<PathBuf> {
        Ok(dirs::cache_dir()
            .ok_or_else(|| anyhow!("failed to get cache dir"))?
            .join("pikaconfig")
            .join("cache")
            .join(self.0))
    }
}

const FILE_STATE: StateType = StateType("output");
const DIR_STATE: StateType = StateType("dirs");

const EPHEMERAL_FILE: StateType = StateType("ephemeral_file");
const EPHEMERAL_DIR: StateType = StateType("ephemeral_dir");

const FILE_CACHE: CacheType = CacheType("files");
const DIR_CACHE: CacheType = CacheType("dirs");

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

fn linked_path<R: LocalStateRoot>(path: &Path, root: R) -> Result<PathBuf> {
    let hash = path_hash(path).with_context(|| format!("unable to make hash of {path:?}"))?;
    Ok(root.path()?.join(hash))
}

fn ephemeral_path<R: LocalStateRoot>(workdir: &Path, filename: &Path, root: R) -> Result<PathBuf> {
    let ephemeral_path = workdir.join(filename);
    let hash = path_hash(&ephemeral_path)
        .with_context(|| format!("failed to make hash of {ephemeral_path:?}"))?;
    Ok(root.path()?.join(hash).join(filename))
}

fn create_file_dir(path: &Path) -> Result<()> {
    let dir = path
        .parent()
        .ok_or_else(|| anyhow!("failed to get {path:?} parent"))?;
    create_dir(dir)
}

fn create_dir(dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dir).with_context(|| format!("failed to create {dir:?}"))
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
        let state = linked_path(&dst, FILE_STATE)
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

impl Module for FileState {
    fn pre_install(&self, _rules: &Rules, _registry: &mut dyn Registry) -> Result<()> {
        create_file_dir(&self.state)
    }
    fn install(&self, _rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        file_util::make_symlink(registry, &self.state, &self.dst)
    }
}

pub struct DirectoryState {
    state: PathBuf,
    dst: PathBuf,
}

impl DirectoryState {
    pub fn new(dst: PathBuf) -> Result<Self> {
        let state = linked_path(&dst, DIR_STATE)
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
    fn pre_install(&self, _rules: &Rules, _registry: &mut dyn Registry) -> Result<()> {
        create_dir(&self.state)
    }
    fn install(&self, _rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        file_util::make_symlink(registry, &self.state, &self.dst)
    }
}

pub struct EphemeralFileState {
    state: PathBuf,
}

impl EphemeralFileState {
    pub fn new(workdir: &Path, filename: &Path) -> Result<Self> {
        let state = ephemeral_path(workdir, filename, EPHEMERAL_FILE).with_context(|| {
            format!("failed to build state path for {workdir:?} with {filename:?}")
        })?;
        Ok(Self { state })
    }
    pub fn path(&self) -> &Path {
        &self.state
    }
}

impl Module for EphemeralFileState {
    fn pre_install(&self, _rules: &Rules, _registry: &mut dyn Registry) -> Result<()> {
        create_file_dir(&self.state)
    }
}

pub struct EphemeralDirState {
    state: PathBuf,
}

impl EphemeralDirState {
    pub fn new(workdir: &Path, filename: &Path) -> Result<Self> {
        let state = ephemeral_path(workdir, filename, EPHEMERAL_DIR).with_context(|| {
            format!("failed to build state path for {workdir:?} with {filename:?}")
        })?;
        Ok(Self { state })
    }
    pub fn path(&self) -> &Path {
        &self.state
    }
}

impl Module for EphemeralDirState {
    fn pre_install(&self, _rules: &Rules, _registry: &mut dyn Registry) -> Result<()> {
        create_dir(&self.state)
    }
}

pub struct EphemeralFileCache(PathBuf);

impl EphemeralFileCache {
    pub fn new(workdir: &Path, filename: &Path) -> Result<Self> {
        let cache = ephemeral_path(workdir, filename, FILE_CACHE).with_context(|| {
            format!("failed to build path for {workdir:?} cache file {filename:?}")
        })?;
        Ok(Self(cache))
    }
    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Module for EphemeralFileCache {
    fn pre_install(&self, _rules: &Rules, _registry: &mut dyn Registry) -> Result<()> {
        create_file_dir(&self.0)
    }
}

pub struct EphemeralDirCache(PathBuf);

impl EphemeralDirCache {
    pub fn new(workdir: &Path, filename: &Path) -> Result<Self> {
        let cache = ephemeral_path(workdir, filename, FILE_CACHE).with_context(|| {
            format!("failed to build path for {workdir:?} cache directory {filename:?}")
        })?;
        Ok(Self(cache))
    }
    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Module for EphemeralDirCache {
    fn pre_install(&self, _rules: &Rules, _registry: &mut dyn Registry) -> Result<()> {
        create_dir(&self.0)
    }
}
