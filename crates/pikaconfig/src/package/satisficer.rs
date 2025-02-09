use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::command;
use crate::process_utils;

pub trait Satisficer {
    fn is_satisfied(&self) -> Result<bool>;
}

impl<T> Satisficer for Option<T>
where
    T: Satisficer,
{
    fn is_satisfied(&self) -> Result<bool> {
        match self {
            Some(satisficer) => satisficer.is_satisfied(),
            None => Ok(false),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields, untagged)]
pub enum DependencySatisficer {
    Command { command: PathBuf },
    AnyCommand { any_command: Vec<PathBuf> },
    AllCommands { all_commands: Vec<PathBuf> },
    File { file: PathBuf },
    AnyFile { any_file: Vec<PathBuf> },
    AllFiles { all_files: Vec<PathBuf> },
    PkgConfig { pkg_config: String },
}

impl Satisficer for DependencySatisficer {
    fn is_satisfied(&self) -> Result<bool> {
        match self {
            DependencySatisficer::Command { command } => command::is_command(command)
                .with_context(|| format!("failed to check if {command:?} is available")),
            DependencySatisficer::AnyCommand { any_command } => {
                for cmd in any_command {
                    if command::is_command(cmd)
                        .with_context(|| format!("failed to check if {cmd:?} is available"))?
                    {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            DependencySatisficer::AllCommands { all_commands } => {
                for cmd in all_commands {
                    if !command::is_command(cmd)
                        .with_context(|| format!("failed to check if {cmd:?} is available"))?
                    {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            DependencySatisficer::File { file } => {
                let path = shellexpand::path::tilde(file);
                Ok(path
                    .try_exists()
                    .with_context(|| format!("failed to check if {file:?} exists"))?)
            }
            DependencySatisficer::AnyFile { any_file } => {
                for path in any_file {
                    if path
                        .try_exists()
                        .with_context(|| format!("failed to check if {path:?} exists"))?
                    {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            DependencySatisficer::AllFiles { all_files } => {
                for path in all_files {
                    if !path
                        .try_exists()
                        .with_context(|| format!("failed to check if {path:?} exists"))?
                    {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            DependencySatisficer::PkgConfig { pkg_config } => {
                let mut cmd = Command::new("pkg-config");
                cmd.arg("--").arg(pkg_config);
                Ok(process_utils::run(&mut cmd).is_ok())
            }
        }
    }
}
