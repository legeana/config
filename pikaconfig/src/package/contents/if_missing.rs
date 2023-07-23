use std::path::Path;
use std::path::PathBuf;

use crate::module::{Module, ModuleBox, Rules};
use crate::registry::Registry;

use super::args::Arguments;
use super::engine;
use super::inventory;
use super::util;

use anyhow::{Context, Result};
use indoc::formatdoc;

struct IfMissing {
    path: PathBuf,
    cmd: ModuleBox,
}

impl IfMissing {
    fn run<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce() -> Result<()>,
    {
        if self
            .path
            .try_exists()
            .with_context(|| format!("failed to check if {:?} exists", &self.path))?
        {
            return Ok(());
        }
        f()
    }
}

impl Module for IfMissing {
    fn pre_install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        self.run(|| self.cmd.pre_install(rules, registry))
    }
    fn install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        self.run(|| self.cmd.install(rules, registry))
    }
    fn post_install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        self.run(|| self.cmd.post_install(rules, registry))
    }
}

#[derive(Debug)]
struct IfMissingStatement {
    path: String,
    cmd: engine::StatementBox,
}

impl engine::Statement for IfMissingStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        let path: PathBuf = shellexpand::tilde(&self.path).as_ref().into();
        match self.cmd.eval(ctx)? {
            Some(cmd) => Ok(Some(Box::new(IfMissing { path, cmd }))),
            None => Ok(None),
        }
    }
}

#[derive(Clone)]
struct IfMissingBuilder;

impl engine::CommandBuilder for IfMissingBuilder {
    fn name(&self) -> String {
        "if_missing".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <path> <command> [<args>...]
                execute a MANIFEST <command> only if <path> is missing
        ", command=self.name()}
    }
    fn build(&self, workdir: &Path, args: &Arguments) -> Result<engine::StatementBox> {
        let (path, cmd_args) = util::multiple_args(&self.name(), args, 1)?;
        assert_eq!(path.len(), 1);
        let cmd_args: Vec<_> = cmd_args.iter().map(String::as_str).collect();
        Ok(Box::new(IfMissingStatement {
            path: path[0].to_owned(),
            cmd: engine::parse_args(workdir, &cmd_args)?,
        }))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(IfMissingBuilder {}));
}
