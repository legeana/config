use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

const PACKAGE_CONFIG_NAME: &str = "package.toml";

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

pub fn load_file(config_path: &Path) -> Result<Package> {
    let raw_input =
        std::fs::read(config_path).with_context(|| format!("failed to read {config_path:?}"))?;
    let input = String::from_utf8(raw_input)
        .with_context(|| format!("failed to convert {config_path:?} to utf8"))?;
    load_string(&input).with_context(|| format!("failed to load {config_path:?}"))
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
}
