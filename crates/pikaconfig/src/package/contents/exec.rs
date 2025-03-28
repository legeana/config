use std::ffi::OsString;
use std::path::{Path, PathBuf};

use anyhow::Result;
use indoc::formatdoc;
use process_utils::Command;
use registry::Registry;

use crate::module::{BoxedModule, Module, Rules};

use super::args::{Argument, Arguments};
use super::engine;
use super::inventory;

#[derive(Debug, Clone, PartialEq)]
enum ExecCondition {
    Always,
    UpdateOnly,
}

struct PostInstallExec {
    exec_condition: ExecCondition,
    current_dir: PathBuf,
    cmd: OsString,
    args: Vec<OsString>,
}

impl Module for PostInstallExec {
    fn post_install(&self, rules: &Rules, _registry: &mut dyn Registry) -> Result<()> {
        if self.exec_condition == ExecCondition::UpdateOnly && !rules.force_update {
            return Ok(());
        }
        Command::new(&self.cmd)
            .args(&self.args)
            .current_dir(&self.current_dir)
            .run()
    }
}

#[derive(Debug)]
struct PostInstallStatement {
    exec_condition: ExecCondition,
    cmd: Argument,
    args: Vec<Argument>,
}

impl engine::Statement for PostInstallStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<BoxedModule>> {
        let args = ctx.expand_args(&self.args)?;
        Ok(Some(Box::new(PostInstallExec {
            exec_condition: self.exec_condition.clone(),
            current_dir: ctx.prefix.clone(),
            cmd: ctx.expand_arg(&self.cmd)?,
            args,
        })))
    }
}

#[derive(Clone)]
struct PostInstallExecBuilder;

impl engine::CommandBuilder for PostInstallExecBuilder {
    fn name(&self) -> String {
        "post_install_exec".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <arg0> [<arg1>...]
                execute a command in a post-install phase
        ", command=self.name()}
    }
    fn build(&self, _workdir: &Path, args: &Arguments) -> Result<engine::Command> {
        let (command, args) = args.expect_at_least_one_arg(self.name())?;
        let cmd = command.clone();
        let args = args.to_vec();
        Ok(engine::Command::new_statement(PostInstallStatement {
            exec_condition: ExecCondition::Always,
            cmd,
            args,
        }))
    }
}

#[derive(Clone)]
struct PostInstallUpdateBuilder;

impl engine::CommandBuilder for PostInstallUpdateBuilder {
    fn name(&self) -> String {
        "post_install_update".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <arg0> [<arg1>...]
                execute a command in a post-install phase
                only if executed via 'setup update' command
        ", command=self.name()}
    }
    fn build(&self, _workdir: &Path, args: &Arguments) -> Result<engine::Command> {
        let (command, args) = args.expect_at_least_one_arg(self.name())?;
        let cmd = command.clone();
        let args = args.to_vec();
        Ok(engine::Command::new_statement(PostInstallStatement {
            exec_condition: ExecCondition::UpdateOnly,
            cmd,
            args,
        }))
    }
}

pub(super) fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(PostInstallExecBuilder));
    registry.register_command(Box::new(PostInstallUpdateBuilder));
}
