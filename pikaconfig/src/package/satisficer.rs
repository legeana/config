use std::path::PathBuf;

use anyhow::Result;

use crate::command;

pub trait Satisficer {
    fn is_satisfied(&self) -> Result<bool>;
}

impl Satisficer for Box<dyn Satisficer> {
    fn is_satisfied(&self) -> Result<bool> {
        self.as_ref().is_satisfied()
    }
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

pub struct WantsCommand {
    command: PathBuf,
}

impl WantsCommand {
    pub fn new(command: impl Into<PathBuf>) -> Self {
        Self { command: command.into() }
    }
}

impl Satisficer for WantsCommand {
    fn is_satisfied(&self) -> Result<bool> {
        command::is_command(&self.command)
    }
}
