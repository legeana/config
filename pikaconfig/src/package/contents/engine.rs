use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

use crate::module::ModuleBox;

use super::args::Arguments;
use super::engine;

pub struct Context {
    pub enabled: bool,
    pub prefix: PathBuf,
}

impl Context {
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

// TODO: rename Parser into something better
/// Parses a Statement.
/// This should be purely syntactical.
pub trait Parser: Sync + Send {
    fn name(&self) -> String;
    fn help(&self) -> String;
    fn parse(&self, workdir: &Path, args: &[&str]) -> Result<StatementBox>;
}

pub type ParserBox = Box<dyn Parser>;

pub trait ConditionBuilder: Sync + Send {
    fn name(&self) -> String;
    fn help(&self) -> String;
    fn build(&self, workdir: &Path, args: &Arguments) -> Result<ConditionBox>;
}

pub type ConditionBuilderBox = Box<dyn ConditionBuilder>;

pub trait Condition: std::fmt::Debug {
    fn eval(&self, ctx: &engine::Context) -> Result<bool>;
}

pub type ConditionBox = Box<dyn Condition>;

/// Command creates a Module or modifies State.
pub trait Statement: std::fmt::Debug {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>>;
}

pub type StatementBox = Box<dyn Statement>;

pub fn parse_args(workdir: &Path, args: &[&str]) -> Result<StatementBox> {
    if args.is_empty() {
        return Err(anyhow!("command with no args[0] should not exist"));
    }
    parse(workdir, args[0], &args[1..])
}

pub fn parse(workdir: &Path, command: &str, args: &[&str]) -> Result<StatementBox> {
    let parser = super::inventory::parser(command)?;
    parser.parse(workdir, args)
}

pub fn new_condition(workdir: &Path, name: &str, args: &Arguments) -> Result<ConditionBox> {
    let builder = super::inventory::condition(name)?;
    builder.build(workdir, args)
}

pub fn help() -> String {
    let mut help = String::new();
    help.push_str("## Commands\n");
    for parser in super::inventory::parsers() {
        help.push_str(parser.help().trim_end());
        help.push('\n');
    }
    help.push('\n');
    help.push_str("## Conditions\n");
    for condition in super::inventory::conditions() {
        help.push_str(condition.help().trim());
        help.push('\n');
    }
    help
}
