use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

const PACKAGE_CONFIG_NAME: &str = "package.toml";

fn default_has_contents() -> bool {
    true
}

/// package.toml file definition
#[derive(Deserialize, PartialEq, Eq, Default, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Package {
    pub name: Option<String>,
    pub requires: Option<Vec<String>>,
    pub conflicts: Option<Vec<String>>,
    #[serde(default = "default_has_contents")]
    pub has_contents: bool,
    pub ansible_playbooks: Option<Vec<AnsiblePlaybook>>,
    pub dependencies: Option<Vec<Dependency>>,
    pub system_dependencies: Option<Vec<SystemDependency>>,
    pub user_dependencies: Option<Vec<UserDependency>>,
}

#[derive(Deserialize, PartialEq, Eq, Default, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Dependency {
    /// Required tags.
    pub requires: Option<Vec<String>>,
    /// Conflicting tags.
    pub conflicts: Option<Vec<String>>,
    pub names: Vec<String>,
}

/// SystemDependency doesn't consider missing package manager a failure.
#[derive(Deserialize, PartialEq, Eq, Default, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct SystemDependency {
    /// Required tags.
    pub requires: Option<Vec<String>>,
    /// Conflicting tags.
    pub conflicts: Option<Vec<String>>,
    // Package managers.
    // It is expected that only one will be available at any time.
    pub any: Option<Vec<String>>,
    pub apt: Option<Vec<String>>,
    pub pacman: Option<Vec<String>>,
    /// Custom multi-line shell script.
    /// Use requires/conflicts for platform selection.
    /// This is intentionally non-portable because arbitrary shell commands
    /// are never portable. It gives a lot of flexibility to write custom
    /// installers.
    pub bash: Option<String>,
}

#[derive(Deserialize, PartialEq, Eq, Default, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct UserDependency {
    /// Required tags.
    pub requires: Option<Vec<String>>,
    /// Conflicting tags.
    pub conflicts: Option<Vec<String>>,
    // User-level package managers.
    pub brew: Option<Vec<String>>,
    pub npm: Option<Vec<String>>,
    pub pip_user: Option<Vec<String>>,
    pub ansible_galaxy_role: Option<Vec<String>>,
    pub ansible_galaxy_collection: Option<Vec<String>>,
}

#[derive(Deserialize, PartialEq, Eq, Default, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct AnsiblePlaybook {
    pub playbook: String,
    pub ask_become_pass: Option<bool>,
}

pub fn load_string(data: &str) -> Result<Package> {
    let deserializer = toml::Deserializer::new(data);
    let pkg = Package::deserialize(deserializer).context("failed to deserialize Package")?;
    Ok(pkg)
}

pub fn load_file(config_path: &Path) -> Result<Package> {
    let raw_input =
        std::fs::read(config_path).with_context(|| format!("failed to read {config_path:?}"))?;
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
        assert_eq!(pkg.requires, None);
        assert_eq!(pkg.conflicts, None);
        assert_eq!(pkg.has_contents, true);
        assert_eq!(pkg.dependencies, None);
        assert_eq!(pkg.system_dependencies, None);
    }

    #[test]
    fn test_load_example() {
        let pkg = load_string(
            "
            name = 'test'
            requires = ['r1', 'r2']
            conflicts = ['c1', 'c2']
            has_contents = false

            [[ansible_playbooks]]
            playbook = 'playbook1.yml'

            [[ansible_playbooks]]
            playbook = 'playbook2.yml'
            ask_become_pass = true

            [[dependencies]]
            names = ['pkg1', 'pkg2']

            [[dependencies]]
            names = ['pkg3']

            [[system_dependencies]]
            any = ['pkg1', 'pkg2']

            [[system_dependencies]]
            apt = ['pkg1-part-deb', 'pkg2-part-deb']

            [[system_dependencies]]
            pacman = ['pkg1-part-arch', 'pkg2-part-arch']

            [[user_dependencies]]
            pip_user = ['pkg1-pip', 'pkg2-pip']
        ",
        )
        .expect("load_string");
        assert_eq!(pkg.name, Some("test".to_owned()));
        assert_eq!(pkg.requires, Some(vec!["r1".to_owned(), "r2".to_owned()]));
        assert_eq!(pkg.conflicts, Some(vec!["c1".to_owned(), "c2".to_owned()]));
        assert_eq!(pkg.has_contents, false);
        assert_eq!(
            pkg.ansible_playbooks,
            Some(vec![
                AnsiblePlaybook {
                    playbook: "playbook1.yml".to_owned(),
                    ..AnsiblePlaybook::default()
                },
                AnsiblePlaybook {
                    playbook: "playbook2.yml".to_owned(),
                    ask_become_pass: Some(true),
                },
            ])
        );
        assert_eq!(
            pkg.dependencies,
            Some(vec![
                Dependency {
                    names: vec!["pkg1".to_owned(), "pkg2".to_owned()],
                    ..Dependency::default()
                },
                Dependency {
                    names: vec!["pkg3".to_owned()],
                    ..Dependency::default()
                }
            ])
        );
        assert_eq!(
            pkg.system_dependencies,
            Some(vec![
                SystemDependency {
                    any: Some(vec!["pkg1".to_owned(), "pkg2".to_owned(),]),
                    ..SystemDependency::default()
                },
                SystemDependency {
                    apt: Some(vec!["pkg1-part-deb".to_owned(), "pkg2-part-deb".to_owned(),]),
                    ..SystemDependency::default()
                },
                SystemDependency {
                    pacman: Some(vec![
                        "pkg1-part-arch".to_owned(),
                        "pkg2-part-arch".to_owned()
                    ]),
                    ..SystemDependency::default()
                },
            ])
        );
        assert_eq!(
            pkg.user_dependencies,
            Some(vec![UserDependency {
                pip_user: Some(vec!["pkg1-pip".to_owned(), "pkg2-pip".to_owned()]),
                ..UserDependency::default()
            },])
        );
    }
}
