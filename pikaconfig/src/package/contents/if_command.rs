use std::path::Path;

use anyhow::Result;
use indoc::formatdoc;

use crate::command::is_command;
use crate::module::{Module, ModuleBox, Rules};
use crate::registry::Registry;

use super::engine;
use super::inventory;
use super::util;

struct IfCommand {
    executable: String,
    cmd: ModuleBox,
}

impl IfCommand {
    fn run<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce() -> Result<()>,
    {
        if !is_command(&self.executable)? {
            return Ok(());
        }
        f()
    }
}

impl Module for IfCommand {
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
struct IfCommandStatement {
    executable: String,
    cmd: engine::StatementBox,
}

impl engine::Statement for IfCommandStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        match self.cmd.eval(ctx)? {
            Some(cmd) => Ok(Some(Box::new(IfCommand {
                executable: self.executable.clone(),
                cmd,
            }))),
            None => Ok(None),
        }
    }
}

#[derive(Clone)]
struct IfCommandBuilder;

impl engine::CommandBuilder for IfCommandBuilder {
    fn name(&self) -> String {
        "if_command".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <executable> <command> [<args>...]
                execute a MANIFEST <command> only if <executable> is available
                in PATH
        ", command=self.name()}
    }
    fn parse(&self, workdir: &Path, args: &[&str]) -> Result<engine::StatementBox> {
        let (exe, cmd_args) = util::multiple_args(&self.name(), args, 1)?;
        assert_eq!(exe.len(), 1);
        Ok(Box::new(IfCommandStatement {
            executable: exe[0].to_owned(),
            cmd: engine::parse_args(workdir, cmd_args)?,
        }))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(IfCommandBuilder {}));
}
