use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use thiserror::Error;

use crate::module::Module;

#[derive(Error, Debug)]
pub enum Error {
    #[error("builder {builder}: unsupported command {command}")]
    UnsupportedCommand { builder: String, command: String },
}

pub struct State {
    pub enabled: bool,
    pub prefix: PathBuf,
}

impl State {
    pub fn new() -> Self {
        Self {
            enabled: true,
            prefix: dirs::home_dir().expect("failed to determine home dir"),
        }
    }
    pub fn dst_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.prefix.join(path)
    }
}

/// Parses a Statement.
/// This should be purely syntactical.
pub trait Parser: Sync + Send {
    fn name(&self) -> String;
    fn help(&self) -> String;
    fn parse(&self, workdir: &Path, args: &[&str]) -> Result<Box<dyn Statement>>;
}

/// Command creates a Module or modifies State.
pub trait Statement: std::fmt::Debug {
    fn eval(&self, state: &mut State) -> Result<Option<Box<dyn Module>>>;
}

pub fn parse(workdir: &Path, args: &[&str]) -> Result<Box<dyn Statement>> {
    if args.is_empty() {
        return Err(anyhow!("command with no args[0] should not exist"));
    }
    let command = args[0];
    let parser = super::inventory::parser(command)?;
    parser.parse(workdir, args)
}

pub fn help() -> String {
    let mut help = String::new();
    for parser in super::inventory::parsers() {
        help.push_str(parser.help().trim_end());
        help.push('\n');
    }
    help
}
