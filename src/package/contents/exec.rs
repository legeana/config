use std::path::PathBuf;
use std::process;

use crate::module::{Module, Rules};
use crate::process_utils;

use super::parser;
use super::util;

use anyhow::Result;

pub struct PostInstallExecBuilder;
pub struct PostInstallUpdateBuilder;

const COMMAND: &str = "post_install_exec";
const UPDATE_COMMAND: &str = "post_install_update";

#[derive(PartialEq)]
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
    fn post_install(
        &self,
        rules: &Rules,
        _registry: &mut dyn crate::registry::Registry,
    ) -> Result<()> {
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

fn parse(
    exec_condition: ExecCondition,
    command_name: &'static str,
    state: &mut parser::State,
    args: &[&str],
) -> Result<Option<Box<dyn Module>>> {
    let (command, args) = util::multiple_args(command_name, args, 1)?;
    assert!(command.len() == 1);
    let args: Vec<String> = args
        .iter()
        .map(shellexpand::tilde)
        .map(String::from)
        .collect();
    Ok(Some(Box::new(PostInstallExec {
        exec_condition,
        current_dir: state.prefix.dst_dir.clone(),
        cmd: command[0].to_owned(),
        args,
    })))
}

impl parser::Builder for PostInstallExecBuilder {
    fn name(&self) -> &'static str {
        COMMAND
    }
    fn help(&self) -> &'static str {
        "post_install_exec <arg0> [<arg1>...]
           execute a command in a post-install phase"
    }
    fn parse(&self, state: &mut parser::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        parse(ExecCondition::Always, COMMAND, state, args)
    }
}

impl parser::Builder for PostInstallUpdateBuilder {
    fn name(&self) -> &'static str {
        UPDATE_COMMAND
    }
    fn help(&self) -> &'static str {
        "post_install_update <arg0> [<arg1>...]
           execute a command in a post-install phase
           only if executed via 'setup update' command"
    }
    fn parse(&self, state: &mut parser::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        parse(ExecCondition::UpdateOnly, UPDATE_COMMAND, state, args)
    }
}
