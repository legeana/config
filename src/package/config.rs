use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

use crate::file_util;
use crate::tag_criteria;

const PACKAGE_CONFIG_TOML: &str = "package.toml";

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

impl tag_criteria::TagCriteria for Package {
    fn requires(&self) -> Option<&[String]> {
        self.requires.as_deref()
    }
    fn conflicts(&self) -> Option<&[String]> {
        self.conflicts.as_deref()
    }
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

impl tag_criteria::TagCriteria for Dependency {
    fn requires(&self) -> Option<&[String]> {
        self.requires.as_deref()
    }
    fn conflicts(&self) -> Option<&[String]> {
        self.conflicts.as_deref()
    }
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

impl tag_criteria::TagCriteria for SystemDependency {
    fn requires(&self) -> Option<&[String]> {
        self.requires.as_deref()
    }
    fn conflicts(&self) -> Option<&[String]> {
        self.conflicts.as_deref()
    }
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

impl tag_criteria::TagCriteria for UserDependency {
    fn requires(&self) -> Option<&[String]> {
        self.requires.as_deref()
    }
    fn conflicts(&self) -> Option<&[String]> {
        self.conflicts.as_deref()
    }
}

fn default_ask_become_pass() -> bool {
    false
}

#[derive(Deserialize, PartialEq, Eq, Default, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct AnsiblePlaybook {
    pub playbooks: Vec<String>,
    #[serde(default = "default_ask_become_pass")]
    pub ask_become_pass: bool,
}

fn load_string_toml(data: &str) -> Result<Package> {
    let deserializer = toml::Deserializer::new(data);
    let pkg = Package::deserialize(deserializer).context("failed to deserialize Package")?;
    Ok(pkg)
}

fn try_load_file_toml(config_path: &Path) -> Result<Option<Package>> {
    let maybe_input = file_util::try_read_to_string(config_path)
        .with_context(|| format!("failed to read {config_path:?}"))?;
    let Some(input) = maybe_input else {
        return Ok(None);
    };
    let pkg =
        load_string_toml(&input).with_context(|| format!("failed to parse {config_path:?}"))?;
    Ok(Some(pkg))
}

fn try_load_package(root: &Path) -> Result<Option<Package>> {
    try_load_file_toml(&root.join(PACKAGE_CONFIG_TOML))
}

pub fn load_package(root: &Path) -> Result<Package> {
    let Some(pkg) = try_load_package(root)? else {
        return Err(anyhow!("{root:?} is not a package"))
    };
    Ok(pkg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_empty_string() {
        let pkg = load_string_toml("").expect("load_string_toml");
        assert_eq!(pkg.name, None);
        assert_eq!(pkg.requires, None);
        assert_eq!(pkg.conflicts, None);
        assert_eq!(pkg.has_contents, true);
        assert_eq!(pkg.dependencies, None);
        assert_eq!(pkg.system_dependencies, None);
    }

    #[test]
    fn test_load_example() {
        let pkg = load_string_toml(
            "
            name = 'test'
            requires = ['r1', 'r2']
            conflicts = ['c1', 'c2']
            has_contents = false

            [[ansible_playbooks]]
            playbooks = ['playbook1.yml']

            [[ansible_playbooks]]
            playbooks = ['playbook2.yml']
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
        .expect("load_string_toml");
        assert_eq!(pkg.name, Some("test".to_owned()));
        assert_eq!(pkg.requires, Some(vec!["r1".to_owned(), "r2".to_owned()]));
        assert_eq!(pkg.conflicts, Some(vec!["c1".to_owned(), "c2".to_owned()]));
        assert_eq!(pkg.has_contents, false);
        assert_eq!(
            pkg.ansible_playbooks,
            Some(vec![
                AnsiblePlaybook {
                    playbooks: vec!["playbook1.yml".to_owned()],
                    ..AnsiblePlaybook::default()
                },
                AnsiblePlaybook {
                    playbooks: vec!["playbook2.yml".to_owned()],
                    ask_become_pass: true,
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
