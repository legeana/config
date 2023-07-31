use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::module::ModuleBox;

use super::args::Arguments;
use super::engine;

pub struct Context {
    pub enabled: bool,
    pub prefix: PathBuf,
    #[allow(dead_code)]
    vars: HashMap<String, OsString>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            enabled: true,
            prefix: dirs::home_dir().expect("failed to determine home dir"),
            vars: HashMap::new(), // Variables are not inherited.
        }
    }
    pub fn subdir(&self, path: impl AsRef<Path>) -> Self {
        Self {
            enabled: true,
            prefix: self.prefix.join(path.as_ref()),
            vars: HashMap::new(), // Variables are not inherited.
        }
    }
    pub fn dst_path(&self, path: impl AsRef<Path>) -> PathBuf {
        self.prefix.join(path)
    }
    /// Expands tilde and environment variables.
    pub fn expand(&self, input: impl AsRef<str>) -> OsString {
        // TODO: maybe use safer prefix expansion.
        match shellexpand::path::tilde(input.as_ref()) {
            std::borrow::Cow::Borrowed(p) => p.as_os_str().to_owned(),
            std::borrow::Cow::Owned(p) => p.into(),
        }
    }
}

/// Builds a Statement.
/// This should be purely syntactical.
pub trait CommandBuilder: Sync + Send {
    fn name(&self) -> String;
    fn help(&self) -> String;
    fn build(&self, workdir: &Path, args: &Arguments) -> Result<StatementBox>;
}

pub type CommandBuilderBox = Box<dyn CommandBuilder>;

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

pub fn new_command(workdir: &Path, command: &str, args: &Arguments) -> Result<StatementBox> {
    let builder = super::inventory::command(command)?;
    builder.build(workdir, args)
}

pub fn new_condition(workdir: &Path, name: &str, args: &Arguments) -> Result<ConditionBox> {
    let builder = super::inventory::condition(name)?;
    builder.build(workdir, args)
}

pub fn help() -> String {
    let mut help = String::new();
    help.push_str("## Commands\n");
    for parser in super::inventory::commands() {
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
