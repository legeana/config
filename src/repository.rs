use std::{collections::VecDeque, fs::DirEntry};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::package::Package;
use crate::registry::Registry;

use anyhow::{anyhow, Context, Result};

pub struct Repository {
    root: PathBuf,
    name: String,
    packages: Vec<Package>,
}

fn read_dir_sorted(path: &Path) -> Result<Vec<DirEntry>> {
    let mut paths = path.read_dir()
        .with_context(|| format!("failed to read directory {path:?}"))?
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("failed to read dir {path:?}"))?;
    paths.sort_by_key(|de| de.file_name());
    Ok(paths)
}

impl Repository {
    pub fn new(root: PathBuf) -> Result<Self> {
        log::debug!("loading repository {root:?}");
        let name: String = root
            .file_name()
            .ok_or_else(|| anyhow!("failed to get {root:?} basename"))?
            .to_string_lossy()
            .into();
        let mut repository = Repository {
            root,
            name,
            packages: Vec::new(),
        };
        let mut queue = VecDeque::from([repository.root.clone()]);
        while let Some(path) = queue.pop_front() {
            if Package::is_package(&path)? {
                log::debug!("loading package {path:?}");
                let inner_path = path.strip_prefix(&repository.root)
                    .with_context(|| format!("unable to strip repository prefix {:?} from package path {path:?}", &repository.root))?;
                let raw_name = format!("{}/{}", &repository.name, inner_path.to_string_lossy());
                let package = Package::new(path.clone(), raw_name)
                    .with_context(|| format!("failed to load {:?}", &path))?;
                log::debug!("successfully loaded package {path:?}");
                repository.packages.push(package);
                continue;
            }
            log::debug!("traversing package collection directory {path:?}");
            for dir in read_dir_sorted(&path)? {
                if dir.path().file_name() == Some(OsStr::new(".git")) {
                    continue;
                }
                let md = std::fs::metadata(dir.path())
                    .with_context(|| format!("failed to read metadata for {:?}", dir.path()))?;
                if !md.is_dir() {
                    continue;
                }
                queue.push_back(dir.path());
            }
        }
        //repository.packages.sort_by(|a, b| a.name().cmp(b.name()));
        log::debug!("successfully loaded repository {:?}", &repository.root);
        Ok(repository)
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn list(&self) -> Vec<String> {
        self.packages.iter().map(|p| p.name().to_string()).collect()
    }
    pub fn pre_install_all(&self) -> Result<()> {
        for package in self.packages.iter() {
            package
                .pre_install()
                .with_context(|| format!("failed to pre-install {}", package.name()))?;
        }
        Ok(())
    }
    pub fn install_all(&self, registry: &mut dyn Registry) -> Result<()> {
        for package in self.packages.iter() {
            package
                .install(registry)
                .with_context(|| format!("failed to install {}", package.name()))?;
        }
        Ok(())
    }
    pub fn post_install_all(&self) -> Result<()> {
        for package in self.packages.iter() {
            package
                .post_install()
                .with_context(|| format!("failed to post-install {}", package.name()))?;
        }
        Ok(())
    }
    pub fn system_install_all(&self, strict: bool) -> Result<()> {
        for package in self.packages.iter() {
            let result = package
                .system_install()
                .with_context(|| format!("failed to system install {}", package.name()));
            match result {
                Ok(_) => (),
                Err(err) => {
                    if strict {
                        return Err(err);
                    } else {
                        log::error!("failed to install {}: {err}", package.name());
                    }
                }
            };
        }
        Ok(())
    }
}
