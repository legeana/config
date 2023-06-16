use std::path::{Path, PathBuf};
use std::process;

use crate::module::{Module, Rules};
use crate::process_utils;

use super::builder;
use super::util;

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

#[derive(Debug)]
struct PostInstallBuilder {
    exec_condition: ExecCondition,
    cmd: String,
    args: Vec<String>,
}

impl builder::Builder for PostInstallBuilder {
    fn build(&self, state: &mut builder::State) -> Result<Option<Box<dyn Module>>> {
        let args: Vec<String> = self
            .args
            .iter()
            .map(shellexpand::tilde)
            .map(String::from)
            .collect();
        Ok(Some(Box::new(PostInstallExec {
            exec_condition: self.exec_condition.clone(),
            current_dir: state.prefix.dst_dir.clone(),
            cmd: self.cmd.clone(),
            args,
        })))
    }
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
    fn parse(&self, _workdir: &Path, args: &[&str]) -> Result<Box<dyn builder::Builder>> {
        let (command, args) = util::multiple_args(&self.name(), args, 1)?;
        assert!(command.len() == 1);
        Ok(Box::new(PostInstallBuilder {
            exec_condition: ExecCondition::Always,
            cmd: command[0].to_owned(),
            args: args.iter().map(|&s| s.to_owned()).collect(),
        }))
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
    fn parse(&self, _workdir: &Path, args: &[&str]) -> Result<Box<dyn builder::Builder>> {
        let (command, args) = util::multiple_args(&self.name(), args, 1)?;
        assert!(command.len() == 1);
        Ok(Box::new(PostInstallBuilder {
            exec_condition: ExecCondition::UpdateOnly,
            cmd: command[0].to_owned(),
            args: args.iter().map(|&s| s.to_owned()).collect(),
        }))
    }
}

pub fn commands() -> Vec<Box<dyn builder::Parser>> {
    vec![
        Box::new(PostInstallExecParser {}),
        Box::new(PostInstallUpdateParser {}),
    ]
}
