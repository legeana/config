use std::path::PathBuf;
use std::process;

use crate::module::{Module, Rules};
use crate::process_utils;

use super::builder;
use super::util;

use anyhow::Result;
use indoc::formatdoc;

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

fn build(
    exec_condition: ExecCondition,
    command_name: &str,
    state: &mut builder::State,
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

#[derive(Clone)]
struct PostInstallExecParser;

impl builder::Parser for PostInstallExecParser {
    fn name(&self) -> String {
        "post_install_exec".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <arg0> [<arg1>...]
                execute a command in a post-install phase
        ", command=self.name()}
    }
    fn build(&self, state: &mut builder::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        build(ExecCondition::Always, &self.name(), state, args)
    }
}

#[derive(Clone)]
struct PostInstallUpdateParser;

impl builder::Parser for PostInstallUpdateParser {
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
    fn build(&self, state: &mut builder::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        build(ExecCondition::UpdateOnly, &self.name(), state, args)
    }
}

pub fn commands() -> Vec<Box<dyn builder::Parser>> {
    vec![
        Box::new(PostInstallExecParser {}),
        Box::new(PostInstallUpdateParser {}),
    ]
}
