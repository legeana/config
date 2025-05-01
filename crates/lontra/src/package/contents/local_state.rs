use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, anyhow};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE;
use lontra_registry::Registry;
use sha2::{Digest as _, Sha256};

use crate::annotated_path::{AnnotatedPath, BoxedAnnotatedPath};
use crate::module::{Module, Rules};

use super::file_util;

fn state_dir() -> Option<PathBuf> {
    if cfg!(unix) {
        if cfg!(target_os = "macos") {
            dirs::data_local_dir()
        } else {
            // The default. Usually Linux.
            dirs::state_dir()
        }
    } else if cfg!(windows) {
        dirs::data_local_dir()
    } else {
        // Unknown, needs fixing.
        None
    }
}

trait LocalStateRoot {
    fn root(&self) -> Result<PathBuf>;
    fn for_linked_path(&self, path: &Path, resource_parts: &[&str]) -> Result<PathBuf> {
        let hash = path_hash(&[path], resource_parts)
            .with_context(|| format!("unable to make hash of {path:?}"))?;
        Ok(self.root()?.join(hash))
    }
    fn for_ephemeral_path(
        &self,
        workdir: &Path,
        filename: &Path,
        resource_id: &str,
    ) -> Result<PathBuf> {
        let ephemeral_path = workdir.join(filename);
        let hash = path_hash(&[&ephemeral_path], &[resource_id])
            .with_context(|| format!("failed to makes hash of {ephemeral_path:?}"))?;
        Ok(self.root()?.join(hash).join(filename))
    }

    // Builders.
    fn linked_dir(&self, link: PathBuf) -> Result<LinkedDir> {
        let path = self
            .for_linked_path(&link, &[])
            .with_context(|| format!("failed to build path for {link:?}"))?;
        Ok(LinkedDir(StateMapping { path, link }))
    }
    fn linked_file(&self, link: PathBuf) -> Result<LinkedFile> {
        let path = self
            .for_linked_path(&link, &[])
            .with_context(|| format!("failed to build state path for {link:?}"))?;
        Ok(LinkedFile(StateMapping { path, link }))
    }
    fn linked_file_cache(&self, link: PathBuf, resource_id: &str) -> Result<LinkedFile> {
        let path = self
            .for_linked_path(&link, &[resource_id])
            .with_context(|| {
                format!("failed to build state path for {link:?} with {resource_id:?}")
            })?;
        Ok(LinkedFile(StateMapping { path, link }))
    }
    fn ephemeral_dir(
        &self,
        workdir: &Path,
        filename: &Path,
        resource_id: &str,
    ) -> Result<EphemeralDir> {
        let path = self
            .for_ephemeral_path(workdir, filename, resource_id)
            .with_context(|| {
                format!("failed to build path for {workdir:?} directory {filename:?}")
            })?;
        Ok(EphemeralDir(path))
    }
    fn ephemeral_file(
        &self,
        workdir: &Path,
        filename: &Path,
        resource_id: &str,
    ) -> Result<EphemeralFile> {
        let path = self
            .for_ephemeral_path(workdir, filename, resource_id)
            .with_context(|| format!("failed to build path for {workdir:?} file {filename:?}"))?;
        Ok(EphemeralFile(path))
    }
}

struct StateType(&'static str);

impl LocalStateRoot for StateType {
    fn root(&self) -> Result<PathBuf> {
        Ok(state_dir()
            .ok_or_else(|| anyhow!("failed to get state dir"))?
            .join("pikaconfig")
            .join(self.0))
    }
}

struct CacheType(&'static str);

impl LocalStateRoot for CacheType {
    fn root(&self) -> Result<PathBuf> {
        Ok(dirs::cache_dir()
            .ok_or_else(|| anyhow!("failed to get cache dir"))?
            .join("pikaconfig")
            .join("cache")
            .join(self.0))
    }
}

fn path_hash(path_parts: &[&Path], resource_parts: &[&str]) -> Result<PathBuf> {
    let mut hasher = Sha256::new();
    let mut first = true;
    let mut update = |bytes: &[u8]| {
        if !first {
            hasher.update([0u8]);
        }
        hasher.update(bytes);
        first = false;
    };

    for path in path_parts {
        let path_str = path
            // TODO: this is not cross-platform.
            // Maybe use to_string_lossy(), os_str_bytes or OsStrExt.
            .to_str()
            .ok_or_else(|| anyhow!("unable to convert {path:?} path to string"))?;
        update(path_str.as_bytes());
    }

    for resource_part in resource_parts {
        update(resource_part.as_bytes());
    }
    let result = hasher.finalize();

    // URL_SAFE is used for compatibility with Python version of pikaconfig.
    Ok(URL_SAFE.encode(result).into())
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

#[derive(Clone, PartialEq)]
struct StateMapping {
    /// The actual file.
    path: PathBuf,
    /// The destination symlink.
    link: PathBuf,
}

impl AnnotatedPath for StateMapping {
    fn as_path(&self) -> &Path {
        &self.path
    }
}

impl std::fmt::Debug for StateMapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} (actual {:?})", self.link, self.path)
    }
}

pub(super) struct EphemeralDir(PathBuf);

impl EphemeralDir {
    pub(super) fn state(&self) -> BoxedAnnotatedPath {
        Box::new(self.0.clone())
    }
}

impl Module for EphemeralDir {
    fn pre_install(&self, _rules: &Rules, _registry: &mut dyn Registry) -> Result<()> {
        create_dir(&self.0)
    }
}

pub(super) struct EphemeralFile(PathBuf);

impl EphemeralFile {
    pub(super) fn state(&self) -> BoxedAnnotatedPath {
        Box::new(self.0.clone())
    }
}

impl Module for EphemeralFile {
    fn pre_install(&self, _rules: &Rules, _registry: &mut dyn Registry) -> Result<()> {
        create_file_dir(&self.0)
    }
}

pub(super) struct LinkedDir(StateMapping);

impl LinkedDir {
    pub(super) fn state(&self) -> BoxedAnnotatedPath {
        Box::new(self.0.clone())
    }
}

impl Module for LinkedDir {
    fn pre_install(&self, _rules: &Rules, _registry: &mut dyn Registry) -> Result<()> {
        create_dir(&self.0.path)
    }
    fn install(&self, _rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        file_util::make_symlink(registry, &self.0.path, &self.0.link)
    }
}

pub(super) struct LinkedFile(StateMapping);

impl LinkedFile {
    pub(super) fn state(&self) -> BoxedAnnotatedPath {
        Box::new(self.0.clone())
    }
}

impl Module for LinkedFile {
    fn pre_install(&self, _rules: &Rules, _registry: &mut dyn Registry) -> Result<()> {
        // Make it possible for another Module to simply open the path.
        create_file_dir(&self.0.path)
    }
    fn install(&self, _rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        file_util::make_symlink(registry, &self.0.path, &self.0.link)
    }
}

// Available directories.
pub(super) fn dir_state(link: PathBuf) -> Result<LinkedDir> {
    StateType("dirs").linked_dir(link)
}

pub(super) fn ephemeral_dir_state(workdir: &Path, resource_id: &str) -> Result<EphemeralDir> {
    StateType("ephemeral_dir").ephemeral_dir(workdir, Path::new("ephemeral_dir_state"), resource_id)
}

pub(super) fn file_state(link: PathBuf) -> Result<LinkedFile> {
    StateType("output").linked_file(link)
}

pub(super) fn dir_cache(
    workdir: &Path,
    filename: &Path,
    resource_id: &str,
) -> Result<EphemeralDir> {
    CacheType("dirs").ephemeral_dir(workdir, filename, resource_id)
}

pub(super) fn file_cache(
    workdir: &Path,
    filename: &Path,
    resource_id: &str,
) -> Result<EphemeralFile> {
    CacheType("files").ephemeral_file(workdir, filename, resource_id)
}

pub(super) fn linked_file_cache(link: PathBuf, resource_id: &str) -> Result<LinkedFile> {
    CacheType("linked_files").linked_file_cache(link, resource_id)
}
