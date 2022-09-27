use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};
use serde::Deserialize;

const PACKAGE_CONFIG_NAME: &str = "package.toml";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("file {0:?} not found")]
    FileNotFound(PathBuf),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

type Result<T> = std::result::Result<T, Error>;

/// package.toml file definition
#[derive(Deserialize, PartialEq, Eq, Default, Debug, Clone)]
pub struct Package {
    pub name: Option<String>,
    pub dependencies: Option<Vec<Dependency>>,
    pub system_dependencies: Option<Vec<SystemDependency>>,
}

#[derive(Deserialize, PartialEq, Eq, Default, Debug, Clone)]
pub struct Dependency {
    pub name: String,
}

#[derive(Deserialize, PartialEq, Eq, Default, Debug, Clone)]
pub struct SystemDependency {
    /// Required tags.
    pub requires: Option<Vec<String>>,
    /// Conflicting tags.
    pub conflicts: Option<Vec<String>>,
    // Do not fail if package manager is not available.
    pub if_available: Option<bool>,
    // Package managers.
    pub apt: Option<Vec<String>>,
    pub brew: Option<Vec<String>>,
    pub npm: Option<Vec<String>>,
    pub pacman: Option<Vec<String>>,
    pub pip_user: Option<Vec<String>>,
    /// Custom multi-line shell script.
    /// Use requires/conflicts for platform selection.
    pub exec: Option<String>,
}

pub fn load_string(data: &str) -> Result<Package> {
    let mut deserializer = toml::Deserializer::new(data);
    let pkg =
        Package::deserialize(&mut deserializer).with_context(|| "failed to deserialize Package")?;
    Ok(pkg)
}

fn read(path: &Path) -> Result<Vec<u8>> {
    match std::fs::read(path) {
        Ok(data) => Ok(data),
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => Err(Error::FileNotFound(path.to_owned())),
            _ => Err(Error::Other(anyhow!("failed to read {path:?}"))),
        },
    }
}

pub fn load_file(config_path: &Path) -> Result<Package> {
    let raw_input = read(config_path)?;
    let input = String::from_utf8(raw_input)
        .with_context(|| format!("failed to convert {config_path:?} to utf8"))?;
    let pkg = load_string(&input).with_context(|| format!("failed to load {config_path:?}"))?;
    Ok(pkg)
}

pub fn load_package(config_path: &Path) -> Result<Package> {
    load_file(&config_path.join(PACKAGE_CONFIG_NAME))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_empty_string() {
        let pkg = load_string("").expect("load_string");
        assert_eq!(pkg.name, None);
        assert_eq!(pkg.dependencies, None);
        assert_eq!(pkg.system_dependencies, None);
    }

    #[test]
    fn test_load_example() {
        let pkg = load_string(
            "
            name = 'test'

            [[dependencies]]
            name = 'pkg1'

            [[dependencies]]
            name = 'pkg2'

            [[system_dependencies]]
            pip_user = ['pkg1-pip', 'pkg2-pip']

            [[system_dependencies]]
            if_available = true
            apt = ['pkg1-part-deb', 'pkg2-part-deb']

            [[system_dependencies]]
            if_available = true
            pacman = ['pkg1-part-arch', 'pkg2-part-arch']
        ",
        )
        .expect("load_string");
        assert_eq!(pkg.name, Some("test".to_owned()));
        assert_eq!(
            pkg.dependencies,
            Some(vec![
                Dependency {
                    name: "pkg1".to_owned(),
                },
                Dependency {
                    name: "pkg2".to_owned(),
                }
            ])
        );
        assert_eq!(
            pkg.system_dependencies,
            Some(vec![
                SystemDependency {
                    pip_user: Some(vec!["pkg1-pip".to_owned(), "pkg2-pip".to_owned()]),
                    ..SystemDependency::default()
                },
                SystemDependency {
                    if_available: Some(true),
                    apt: Some(vec!["pkg1-part-deb".to_owned(), "pkg2-part-deb".to_owned(),]),
                    ..SystemDependency::default()
                },
                SystemDependency {
                    if_available: Some(true),
                    pacman: Some(vec![
                        "pkg1-part-arch".to_owned(),
                        "pkg2-part-arch".to_owned()
                    ]),
                    ..SystemDependency::default()
                },
            ])
        );
    }
}
