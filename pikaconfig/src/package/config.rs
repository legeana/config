#![allow(clippy::bool_assert_comparison)]

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

use crate::file_util;
use crate::iter_util;
use crate::tag_criteria;

use super::satisficer::DependencySatisficer;

const PACKAGE_CONFIG_TOML: &str = "package.toml";
const PACKAGE_CONFIG_YAML: &str = "package.yaml";

fn default_has_contents() -> bool {
    true
}

/// package.toml file definition
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Package {
    pub name: Option<String>,
    pub requires: Option<tag_criteria::TagCriteria>,
    #[serde(default = "default_has_contents")]
    pub has_contents: bool,
    pub dependencies: Option<Vec<Dependency>>,
    pub system_dependencies: Option<Vec<SystemDependency>>,
    pub user_dependencies: Option<Vec<UserDependency>>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Dependency {
    pub requires: Option<tag_criteria::TagCriteria>,
    pub names: Vec<String>,
}

/// SystemDependency doesn't consider missing package manager a failure.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SystemDependency {
    pub requires: Option<tag_criteria::TagCriteria>,
    // Package managers.
    // It is expected that only one will be available at any time.
    pub any: Option<Vec<String>>,
    pub apt: Option<Vec<String>>,
    pub pacman: Option<Vec<String>>,
    pub winget: Option<Vec<String>>,
    /// Satisfaction criteria.
    /// Will skip this dependency if met.
    /// Rules::force_download will force this dependency to be updated.
    pub wants: Option<DependencySatisficer>,
    /// Custom multi-line shell script.
    /// Use requires for platform selection.
    /// This is intentionally non-portable because arbitrary shell commands
    /// are never portable. It gives a lot of flexibility to write custom
    /// installers.
    pub bash: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BrewConfig {
    pub taps: Option<Vec<String>>,
    pub casks: Option<Vec<String>>,
    pub formulas: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields, untagged)]
pub enum BrewDependency {
    Formulas(Vec<String>),
    Config(BrewConfig),
}

impl BrewDependency {
    pub fn to_config(&self) -> BrewConfig {
        match self {
            BrewDependency::Formulas(formulas) => BrewConfig {
                formulas: Some(formulas.clone()),
                ..Default::default()
            },
            BrewDependency::Config(config) => config.clone(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields, untagged)]
pub enum CargoDependency {
    Crates(Vec<String>),
    Config {
        crates: Option<Vec<String>>,
        git: Option<String>,
        tag: Option<String>,
        branch: Option<String>,
        path: Option<PathBuf>,
        locked: Option<bool>,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BinaryUrlDependency {
    pub url: String,
    pub filename: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GithubReleaseDependency {
    pub owner: String,
    pub repo: String,
    // Latest if not specified.
    pub release: Option<String>,
    pub asset: String,
    // Defaults to asset name if not specified.
    pub filename: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct UserDependency {
    pub requires: Option<tag_criteria::TagCriteria>,
    /// Satisfaction criteria.
    /// Will skip this dependency if met.
    /// Rules::force_download will force this dependency to be updated.
    pub wants: Option<DependencySatisficer>,
    // User-level package managers.
    pub brew: Option<BrewDependency>,
    pub cargo: Option<CargoDependency>,
    pub npm: Option<Vec<String>>,
    pub pip_user: Option<Vec<String>>,
    // Binary management.
    pub binary_url: Option<BinaryUrlDependency>,
    pub github_release: Option<GithubReleaseDependency>,
}

fn load_toml_string(data: &str) -> Result<Package> {
    toml::from_str(data).context("failed to deserialize Package")
}

fn load_yaml_string(data: &str) -> Result<Package> {
    serde_yaml::from_str(data).context("failed to deserialize Package")
}

fn load_toml_file(config_path: &Path) -> Result<Package> {
    let input = std::fs::read_to_string(config_path)
        .with_context(|| format!("failed to read {config_path:?}"))?;
    load_toml_string(&input).with_context(|| format!("failed to parse {config_path:?}"))
}

fn load_yaml_file(config_path: &Path) -> Result<Package> {
    let input = std::fs::read_to_string(config_path)
        .with_context(|| format!("failed to read {config_path:?}"))?;
    load_yaml_string(&input).with_context(|| format!("failed to parse {config_path:?}"))
}

pub fn load_package(root: &Path) -> Result<Package> {
    let packages: Vec<Package> = [
        file_util::skip_not_found(load_toml_file(&root.join(PACKAGE_CONFIG_TOML)))?,
        file_util::skip_not_found(load_yaml_file(&root.join(PACKAGE_CONFIG_YAML)))?,
    ]
    .into_iter()
    .flatten()
    .collect();
    iter_util::to_option(packages)
        .with_context(|| format!("{root:?} has multiple package.* files"))?
        .ok_or_else(|| anyhow!("{root:?} is not a package"))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_load_empty_string() {
        let pkg = load_toml_string("").expect("load_toml_string");
        assert_eq!(pkg.name, None);
        assert_eq!(pkg.requires, None);
        assert_eq!(pkg.has_contents, true);
        assert_eq!(pkg.dependencies, None);
        assert_eq!(pkg.system_dependencies, None);
    }

    #[test]
    fn test_load_example() {
        let pkg = load_toml_string(
            "
            name = 'test'
            requires = ['r1', 'r2']
            has_contents = false

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

            [[system_dependencies]]
            any = ['pkg']
            wants = { command = 'pkg' }

            [[user_dependencies]]
            pip_user = ['pkg1-pip', 'pkg2-pip']

            [[user_dependencies]]
            pip_user = ['pkg']
            wants = {command = 'pkg-cmd'}

            [[user_dependencies]]
            cargo = ['pkg1', 'pkg2']

            [[user_dependencies]]
            cargo = { git = 'https://github.com/example/project.git' }

            [[user_dependencies]]
            brew = ['pkg1', 'pkg2']

            [[user_dependencies]]
            [user_dependencies.brew]
            taps = ['tap1', 'tap2']
            casks = ['cask1', 'cask2']
            formulas = ['formula1', 'formula2']
        ",
        )
        .expect("load_toml_string");
        assert_eq!(pkg.name, Some("test".to_owned()));
        assert_eq!(
            pkg.requires,
            Some(tag_criteria::TagCriteria::Requires(vec![
                "r1".to_owned(),
                "r2".to_owned()
            ]))
        );
        assert_eq!(pkg.has_contents, false);
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
                SystemDependency {
                    any: Some(vec!["pkg".to_owned()]),
                    wants: Some(DependencySatisficer::Command {
                        command: "pkg".into()
                    }),
                    ..SystemDependency::default()
                },
            ])
        );
        assert_eq!(
            pkg.user_dependencies,
            Some(vec![
                UserDependency {
                    pip_user: Some(vec!["pkg1-pip".to_owned(), "pkg2-pip".to_owned()]),
                    ..UserDependency::default()
                },
                UserDependency {
                    pip_user: Some(vec!["pkg".to_owned()]),
                    wants: Some(DependencySatisficer::Command {
                        command: "pkg-cmd".into()
                    }),
                    ..UserDependency::default()
                },
                UserDependency {
                    cargo: Some(CargoDependency::Crates(vec![
                        "pkg1".to_owned(),
                        "pkg2".to_owned()
                    ])),
                    ..Default::default()
                },
                UserDependency {
                    cargo: Some(CargoDependency::Config {
                        crates: None,
                        git: Some("https://github.com/example/project.git".to_owned()),
                        branch: None,
                        tag: None,
                        path: None,
                        locked: None,
                    }),
                    ..Default::default()
                },
                UserDependency {
                    brew: Some(BrewDependency::Formulas(vec![
                        "pkg1".to_owned(),
                        "pkg2".to_owned()
                    ])),
                    ..Default::default()
                },
                UserDependency {
                    brew: Some(BrewDependency::Config(BrewConfig {
                        taps: Some(vec!["tap1".to_owned(), "tap2".to_owned()]),
                        casks: Some(vec!["cask1".to_owned(), "cask2".to_owned()]),
                        formulas: Some(vec!["formula1".to_owned(), "formula2".to_owned()]),
                    })),
                    ..Default::default()
                },
            ])
        );
    }

    #[test]
    fn load_binary_url_dependency() {
        let pkg = load_toml_string(
            "
            [[user_dependencies]]
            [user_dependencies.binary_url]
            url = 'https://example.com/file.bin'
            filename = 'file.bin'
        ",
        )
        .expect("load_toml_string");
        assert_eq!(
            pkg.user_dependencies,
            Some(vec![UserDependency {
                binary_url: Some(BinaryUrlDependency {
                    url: "https://example.com/file.bin".to_owned(),
                    filename: "file.bin".to_owned(),
                }),
                ..Default::default()
            }]),
        );
    }

    #[test]
    fn load_github_dependency_minimal() {
        let pkg = load_toml_string(
            "
            [[user_dependencies]]
            [user_dependencies.github_release]
            owner = 'owner'
            repo = 'repo'
            asset = 'asset'
        ",
        )
        .expect("load_toml_string");
        assert_eq!(
            pkg.user_dependencies,
            Some(vec![UserDependency {
                github_release: Some(GithubReleaseDependency {
                    owner: "owner".to_owned(),
                    repo: "repo".to_owned(),
                    release: None,
                    asset: "asset".to_owned(),
                    filename: None,
                }),
                ..Default::default()
            }]),
        );
    }

    #[test]
    fn load_github_dependency_full() {
        let pkg = load_toml_string(
            "
            [[user_dependencies]]
            [user_dependencies.github_release]
            owner = 'owner'
            repo = 'repo'
            release = '1.2.3'
            asset = 'asset'
            filename = 'filename'
        ",
        )
        .expect("load_toml_string");
        assert_eq!(
            pkg.user_dependencies,
            Some(vec![UserDependency {
                github_release: Some(GithubReleaseDependency {
                    owner: "owner".to_owned(),
                    repo: "repo".to_owned(),
                    release: Some("1.2.3".to_owned()),
                    asset: "asset".to_owned(),
                    filename: Some("filename".to_owned()),
                }),
                ..Default::default()
            }]),
        );
    }
}
