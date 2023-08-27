use std::path::PathBuf;

use serde::Deserialize;

use anyhow::Result;

use crate::command;

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
}

impl Satisficer for DependencySatisficer {
    fn is_satisfied(&self) -> Result<bool> {
        match self {
            DependencySatisficer::Command { command } => command::is_command(command),
        }
    }
}
