#![allow(clippy::bool_assert_comparison)]

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::string_list::StringList;
use crate::tag_criteria;

use super::satisficer::DependencySatisficer;

const PACKAGE_CONFIG_TOML: &str = "package.toml";

fn default_has_contents() -> bool {
    true
}

/// package.toml file definition
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct Package {
    pub name: Option<String>,
    pub requires: Option<tag_criteria::TagCriteria>,
    #[serde(default = "default_has_contents")]
    pub has_contents: bool,
    pub dependencies: Option<Vec<Dependency>>,
    pub system_dependencies: Option<Vec<SystemDependency>>,
    pub user_dependencies: Option<Vec<UserDependency>>,
}

impl Default for Package {
    #[allow(clippy::default_trait_access)]
    fn default() -> Self {
        Self {
            name: Default::default(),
            requires: Default::default(),
            // Is there a way to override this inside the struct?
            has_contents: default_has_contents(),
            dependencies: Default::default(),
            system_dependencies: Default::default(),
            user_dependencies: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct Dependency {
    pub requires: Option<tag_criteria::TagCriteria>,
    pub names: StringList,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct WingetConfig {
    pub packages: StringList,
    pub source: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields, untagged)]
pub(super) enum WingetDependency {
    WingetSource(StringList),
    Config(WingetConfig),
}

impl WingetDependency {
    pub(super) fn to_config(&self) -> WingetConfig {
        match self {
            Self::WingetSource(p) => WingetConfig {
                packages: p.clone(),
                source: "winget".to_owned(),
            },
            Self::Config(cfg) => cfg.clone(),
        }
    }
}

/// SystemDependency doesn't consider missing package manager a failure.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct SystemDependency {
    pub requires: Option<tag_criteria::TagCriteria>,
    // Package managers.
    // It is expected that only one will be available at any time.
    pub any: Option<StringList>,
    pub apt: Option<StringList>,
    pub pacman: Option<StringList>,
    pub winget: Option<WingetDependency>,
    /// Satisfaction criteria.
    /// Will skip this dependency if met.
    /// Rules::force_update will force this dependency to be updated.
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
pub(super) struct BrewConfig {
    pub taps: Option<StringList>,
    pub casks: Option<StringList>,
    pub formulas: Option<StringList>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields, untagged)]
pub(super) enum BrewDependency {
    Formulas(StringList),
    Config(BrewConfig),
}

impl BrewDependency {
    pub(super) fn to_config(&self) -> BrewConfig {
        match self {
            Self::Formulas(formulas) => BrewConfig {
                formulas: Some(formulas.clone()),
                ..Default::default()
            },
            Self::Config(config) => config.clone(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct CargoConfig {
    pub crates: Option<StringList>,
    pub git: Option<String>,
    pub tag: Option<String>,
    pub branch: Option<String>,
    pub path: Option<PathBuf>,
    pub locked: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields, untagged)]
pub(super) enum CargoDependency {
    Crates(StringList),
    Config(CargoConfig),
}

impl CargoDependency {
    pub(super) fn to_cargo_config(&self) -> CargoConfig {
        match self {
            Self::Crates(crates) => CargoConfig {
                crates: Some(crates.clone()),
                ..Default::default()
            },
            Self::Config(config) => config.clone(),
        }
    }
    #[allow(dead_code)]
    pub(super) fn into_cargo_config(self) -> CargoConfig {
        match self {
            Self::Crates(crates) => CargoConfig {
                crates: Some(crates),
                ..Default::default()
            },
            Self::Config(config) => config,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct FlatpakDependency {
    pub repository: String,
    pub package: String,
    pub alias: Option<String>,
    pub overrides: Option<StringList>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct BinaryUrlDependency {
    pub url: String,
    pub filename: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct GithubReleaseDependency {
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
pub(super) struct UserDependency {
    pub requires: Option<tag_criteria::TagCriteria>,
    /// Satisfaction criteria.
    /// Will skip this dependency if met.
    /// Rules::force_update will force this dependency to be updated.
    pub wants: Option<DependencySatisficer>,
    // User-level package managers.
    pub brew: Option<BrewDependency>,
    pub cargo: Option<CargoDependency>,
    pub npm: Option<StringList>,
    pub pipx: Option<StringList>,
    pub pipx_bootstrap: Option<StringList>,
    pub flatpak: Option<FlatpakDependency>,
    // Binary management.
    pub binary_url: Option<BinaryUrlDependency>,
    pub github_release: Option<GithubReleaseDependency>,
}

fn load_toml_string(data: &str) -> Result<Package> {
    toml::from_str(data).context("failed to deserialize Package")
}

fn load_toml_file(config_path: &Path) -> Result<Package> {
    let input = std::fs::read_to_string(config_path)
        .with_context(|| format!("failed to read {config_path:?}"))?;
    load_toml_string(&input).with_context(|| format!("failed to parse {config_path:?}"))
}

pub(super) fn load_package(root: &Path) -> Result<Package> {
    load_toml_file(&root.join(PACKAGE_CONFIG_TOML))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_load_empty_string() {
        let pkg = load_toml_string("").expect("load_toml_string");
        assert_eq!(
            pkg,
            Package {
                has_contents: true,
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_load_header() {
        let pkg = load_toml_string(
            "
            name = 'test'
            requires = ['r1', 'r2']
            has_contents = false
            ",
        )
        .expect("load_toml_string");
        assert_eq!(
            pkg,
            Package {
                name: Some("test".to_owned()),
                requires: Some(tag_criteria::TagCriteria(StringList::List(vec![
                    "r1".to_owned(),
                    "r2".to_owned(),
                ]))),
                has_contents: false,
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_load_dependencies() {
        let pkg = load_toml_string(
            "
            [[dependencies]]
            names = ['pkg1', 'pkg2']

            [[dependencies]]
            names = ['pkg3']
            ",
        )
        .expect("load_toml_string");
        assert_eq!(
            pkg,
            Package {
                dependencies: Some(vec![
                    Dependency {
                        names: StringList::List(vec!["pkg1".to_owned(), "pkg2".to_owned()]),
                        ..Default::default()
                    },
                    Dependency {
                        names: StringList::List(vec!["pkg3".to_owned()]),
                        ..Default::default()
                    }
                ]),
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_load_system_any() {
        let pkg = load_toml_string(
            "
            [[system_dependencies]]
            any = ['pkg1', 'pkg2']
            ",
        )
        .expect("load_toml_string");
        assert_eq!(
            pkg,
            Package {
                system_dependencies: Some(vec![SystemDependency {
                    any: Some(StringList::List(
                        vec!["pkg1".to_owned(), "pkg2".to_owned(),]
                    )),
                    ..Default::default()
                },]),
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_load_system_apt() {
        let pkg = load_toml_string(
            "
            [[system_dependencies]]
            apt = ['pkg1-part-deb', 'pkg2-part-deb']
            ",
        )
        .expect("load_toml_string");
        assert_eq!(
            pkg,
            Package {
                system_dependencies: Some(vec![SystemDependency {
                    apt: Some(StringList::List(vec![
                        "pkg1-part-deb".to_owned(),
                        "pkg2-part-deb".to_owned(),
                    ])),
                    ..Default::default()
                },]),
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_load_system_pacman() {
        let pkg = load_toml_string(
            "
            [[system_dependencies]]
            pacman = ['pkg1-part-arch', 'pkg2-part-arch']
            ",
        )
        .expect("load_toml_string");
        assert_eq!(
            pkg,
            Package {
                system_dependencies: Some(vec![SystemDependency {
                    pacman: Some(StringList::List(vec![
                        "pkg1-part-arch".to_owned(),
                        "pkg2-part-arch".to_owned()
                    ])),
                    ..Default::default()
                },]),
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_load_system_wants() {
        let pkg = load_toml_string(
            "
            [[system_dependencies]]
            wants = { command = 'pkg' }
            ",
        )
        .expect("load_toml_string");
        assert_eq!(
            pkg.system_dependencies,
            Some(vec![SystemDependency {
                wants: Some(DependencySatisficer::Command {
                    command: "pkg".into()
                }),
                ..Default::default()
            },])
        );
    }

    #[test]
    fn test_load_user_wants() {
        let pkg = load_toml_string(
            "
            [[user_dependencies]]
            wants = { command = 'pkg-cmd' }
            ",
        )
        .expect("load_toml_string");
        assert_eq!(
            pkg,
            Package {
                user_dependencies: Some(vec![UserDependency {
                    wants: Some(DependencySatisficer::Command {
                        command: "pkg-cmd".into()
                    }),
                    ..Default::default()
                },]),
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_load_user_cargo_crates() {
        let pkg = load_toml_string(
            "
            [[user_dependencies]]
            cargo = ['pkg1', 'pkg2']
            ",
        )
        .expect("load_toml_string");
        assert_eq!(
            pkg,
            Package {
                user_dependencies: Some(vec![UserDependency {
                    cargo: Some(CargoDependency::Crates(StringList::List(vec![
                        "pkg1".to_owned(),
                        "pkg2".to_owned(),
                    ]))),
                    ..Default::default()
                },]),
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_load_user_cargo_git() {
        let pkg = load_toml_string(
            "
            [[user_dependencies]]
            cargo = { git = 'https://github.com/example/project.git' }
            ",
        )
        .expect("load_toml_string");
        assert_eq!(
            pkg,
            Package {
                user_dependencies: Some(vec![UserDependency {
                    cargo: Some(CargoDependency::Config(CargoConfig {
                        crates: None,
                        git: Some("https://github.com/example/project.git".to_owned()),
                        branch: None,
                        tag: None,
                        path: None,
                        locked: None,
                    })),
                    ..Default::default()
                },]),
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_load_user_brew_formulas() {
        let pkg = load_toml_string(
            "
            [[user_dependencies]]
            brew = ['pkg1', 'pkg2']
            ",
        )
        .expect("load_toml_string");
        assert_eq!(
            pkg,
            Package {
                user_dependencies: Some(vec![UserDependency {
                    brew: Some(BrewDependency::Formulas(StringList::List(vec![
                        "pkg1".to_owned(),
                        "pkg2".to_owned(),
                    ]))),
                    ..Default::default()
                },]),
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_load_user_brew_config() {
        let pkg = load_toml_string(
            "
            [[user_dependencies]]
            [user_dependencies.brew]
            taps = ['tap1', 'tap2']
            casks = ['cask1', 'cask2']
            formulas = ['formula1', 'formula2']
            ",
        )
        .expect("load_toml_string");
        assert_eq!(
            pkg,
            Package {
                user_dependencies: Some(vec![UserDependency {
                    brew: Some(BrewDependency::Config(BrewConfig {
                        taps: Some(StringList::List(vec!["tap1".to_owned(), "tap2".to_owned()])),
                        casks: Some(StringList::List(vec![
                            "cask1".to_owned(),
                            "cask2".to_owned()
                        ])),
                        formulas: Some(StringList::List(vec![
                            "formula1".to_owned(),
                            "formula2".to_owned()
                        ])),
                    })),
                    ..Default::default()
                },]),
                ..Default::default()
            },
        );
    }

    #[test]
    fn load_flatpak_dependency() {
        let pkg = load_toml_string(
            "
            [[user_dependencies]]
            [user_dependencies.flatpak]
            repository = 'flathub'
            package = 'com.pkg'
            alias = 'pkg'
            overrides = ['--env=foo=bar']
            ",
        )
        .expect("load_toml_string");
        assert_eq!(
            pkg,
            Package {
                user_dependencies: Some(vec![UserDependency {
                    flatpak: Some(FlatpakDependency {
                        repository: "flathub".to_owned(),
                        package: "com.pkg".to_owned(),
                        alias: Some("pkg".to_owned()),
                        overrides: Some(StringList::List(vec!["--env=foo=bar".to_owned()])),
                    }),
                    ..Default::default()
                }]),
                ..Default::default()
            },
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
            pkg,
            Package {
                user_dependencies: Some(vec![UserDependency {
                    binary_url: Some(BinaryUrlDependency {
                        url: "https://example.com/file.bin".to_owned(),
                        filename: "file.bin".to_owned(),
                    }),
                    ..Default::default()
                }]),
                ..Default::default()
            },
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
            pkg,
            Package {
                user_dependencies: Some(vec![UserDependency {
                    github_release: Some(GithubReleaseDependency {
                        owner: "owner".to_owned(),
                        repo: "repo".to_owned(),
                        release: None,
                        asset: "asset".to_owned(),
                        filename: None,
                    }),
                    ..Default::default()
                }]),
                ..Default::default()
            },
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
            pkg,
            Package {
                user_dependencies: Some(vec![UserDependency {
                    github_release: Some(GithubReleaseDependency {
                        owner: "owner".to_owned(),
                        repo: "repo".to_owned(),
                        release: Some("1.2.3".to_owned()),
                        asset: "asset".to_owned(),
                        filename: Some("filename".to_owned()),
                    }),
                    ..Default::default()
                }]),
                ..Default::default()
            },
        );
    }

    #[test]
    fn load_pipx_dependency() {
        let pkg = load_toml_string(
            "
            [[user_dependencies]]
            pipx = ['pkg1', 'pkg2']
            ",
        )
        .expect("load_toml_string");
        assert_eq!(
            pkg,
            Package {
                user_dependencies: Some(vec![UserDependency {
                    pipx: Some(StringList::List(vec!["pkg1".to_owned(), "pkg2".to_owned()])),
                    ..Default::default()
                }]),
                ..Default::default()
            },
        );
    }

    #[test]
    fn load_pipx_bootstrap_dependency() {
        let pkg = load_toml_string(
            "
            [[user_dependencies]]
            pipx_bootstrap = ['pkg1', 'pkg2']
            ",
        )
        .expect("load_toml_string");
        assert_eq!(
            pkg,
            Package {
                user_dependencies: Some(vec![UserDependency {
                    pipx_bootstrap: Some(StringList::List(vec![
                        "pkg1".to_owned(),
                        "pkg2".to_owned()
                    ])),
                    ..Default::default()
                }]),
                ..Default::default()
            },
        );
    }

    #[test]
    fn load_winget_dependency() {
        let pkg = load_toml_string(
            "
            [[system_dependencies]]
            winget = ['pkg1', 'pkg2']
            ",
        )
        .expect("load_toml_string");
        assert_eq!(
            pkg,
            Package {
                system_dependencies: Some(vec![SystemDependency {
                    winget: Some(WingetDependency::WingetSource(StringList::List(vec![
                        "pkg1".to_owned(),
                        "pkg2".to_owned(),
                    ]))),
                    ..Default::default()
                }]),
                ..Default::default()
            },
        );
    }

    #[test]
    fn load_winget_dependency_config() {
        let pkg = load_toml_string(
            "
            [[system_dependencies]]
            winget = { packages = ['pkg1', 'pkg2'], source = 'msstore' }
            ",
        )
        .expect("load_toml_string");
        assert_eq!(
            pkg,
            Package {
                system_dependencies: Some(vec![SystemDependency {
                    winget: Some(WingetDependency::Config(WingetConfig {
                        packages: StringList::List(vec!["pkg1".to_owned(), "pkg2".to_owned()]),
                        source: "msstore".to_owned(),
                    })),
                    ..Default::default()
                }]),
                ..Default::default()
            },
        );
    }

    #[test]
    fn winget_dependency_source_to_config() {
        let w = WingetDependency::WingetSource(StringList::Single("pkg".to_owned()));
        assert_eq!(
            w.to_config(),
            WingetConfig {
                packages: StringList::Single("pkg".to_owned()),
                source: "winget".to_owned(),
            },
        );
    }

    #[test]
    fn winget_dependency_config_to_config() {
        let cfg = WingetConfig {
            packages: StringList::Single("pkg".to_owned()),
            source: "winget".to_owned(),
        };
        let w = WingetDependency::Config(cfg.clone());

        assert_eq!(w.to_config(), cfg);
    }

    #[test]
    fn cargo_dependency_crates_to_cargo_config() {
        let c = CargoDependency::Crates(StringList::Single("pkg".to_owned()));
        assert_eq!(
            c.to_cargo_config(),
            CargoConfig {
                crates: Some(StringList::Single("pkg".to_owned())),
                ..Default::default()
            },
        );
    }

    #[test]
    fn cargo_dependency_config_to_cargo_config() {
        let cfg = CargoConfig {
            crates: Some(StringList::List(vec![
                "crate-1".to_owned(),
                "crate-2".to_owned(),
            ])),
            git: Some("https://example.com/some.git".to_owned()),
            ..Default::default()
        };
        let c = CargoDependency::Config(cfg.clone());

        assert_eq!(c.to_cargo_config(), cfg);
    }
}
