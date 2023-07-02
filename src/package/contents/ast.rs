use std::path::Path;

use anyhow::{anyhow, Result};

use crate::module::ModuleBox;

use super::engine;

/// Parses a Statement.
/// This should be purely syntactical.
pub trait Parser: Sync + Send {
    fn name(&self) -> String;
    fn help(&self) -> String;
    fn parse(&self, workdir: &Path, args: &[&str]) -> Result<StatementBox>;
}

pub type ParserBox = Box<dyn Parser>;

/// Command creates a Module or modifies State.
pub trait Statement: std::fmt::Debug {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>>;
}

pub type StatementBox = Box<dyn Statement>;

pub fn parse(workdir: &Path, args: &[&str]) -> Result<StatementBox> {
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
