use std::path::{Path, PathBuf};
use std::process;

use crate::module::{Module, ModuleBox, Rules};
use crate::process_utils;
use crate::registry::Registry;

use super::args::Arguments;
use super::engine;
use super::inventory;

use anyhow::Result;
use indoc::formatdoc;

#[derive(Debug, Clone, PartialEq)]
enum ExecCondition {
    Always,
    UpdateOnly,
}

struct PostInstallExec {
    exec_condition: ExecCondition,
    current_dir: PathBuf,
    cmd: String,
    args: Vec<String>,
}

impl Module for PostInstallExec {
    fn post_install(&self, rules: &Rules, _registry: &mut dyn Registry) -> Result<()> {
        if self.exec_condition == ExecCondition::UpdateOnly && !rules.force_download {
            return Ok(());
        }
        process_utils::run(
            process::Command::new(&self.cmd)
                .args(&self.args)
                .current_dir(&self.current_dir),
        )
    }
}

#[derive(Debug)]
struct PostInstallStatement {
    exec_condition: ExecCondition,
    cmd: String,
    args: Vec<String>,
}

impl engine::Statement for PostInstallStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        let args: Vec<String> = self
            .args
            .iter()
            .map(shellexpand::tilde)
            .map(String::from)
            .collect();
        Ok(Some(Box::new(PostInstallExec {
            exec_condition: self.exec_condition.clone(),
            current_dir: ctx.prefix.clone(),
            cmd: self.cmd.clone(),
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
    fn build(&self, _workdir: &Path, args: &Arguments) -> Result<engine::StatementBox> {
        let (command, args) = args.expect_variadic_args(self.name(), 1)?;
        assert!(command.len() == 1);
        Ok(Box::new(PostInstallStatement {
            exec_condition: ExecCondition::Always,
            cmd: command[0].to_owned(),
            args: args.to_vec(),
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
    fn build(&self, _workdir: &Path, args: &Arguments) -> Result<engine::StatementBox> {
        let (command, args) = args.expect_variadic_args(self.name(), 1)?;
        assert!(command.len() == 1);
        Ok(Box::new(PostInstallStatement {
            exec_condition: ExecCondition::UpdateOnly,
            cmd: command[0].to_owned(),
            args: args.to_vec(),
        }))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(PostInstallExecBuilder {}));
    registry.register_command(Box::new(PostInstallUpdateBuilder {}));
}
