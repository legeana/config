use std::path::PathBuf;
use std::process;

use crate::package::configuration::parser;
use crate::package::configuration::util::multiple_args;
use crate::package::configuration::Configuration;

use anyhow::{anyhow, Context, Result};

pub struct PostInstallExecParser;

const COMMAND: &str = "post_install_exec";

struct PostInstallExecHook {
    current_dir: PathBuf,
    cmd: String,
    args: Vec<String>,
}

impl super::Hook for PostInstallExecHook {
    fn execute(&self) -> Result<()> {
        let status = process::Command::new(&self.cmd)
            .args(&self.args)
            .current_dir(&self.current_dir)
            .status()
            .with_context(|| format!("failed to start {}", self.cmd))?;
        if !status.success() {
            return Err(anyhow!(
                "failed to execute {:?} $ {} {:?}",
                self.current_dir,
                self.cmd,
                self.args,
            ));
        }
        Ok(())
    }
}

impl parser::Parser for PostInstallExecParser {
    fn name(&self) -> &'static str {
        COMMAND
    }
    fn help(&self) -> &'static str {
        "post_install_exec <arg0> [<arg1>...]
           execute a command in a post-install phase"
    }
    fn parse(
        &self,
        state: &mut parser::State,
        configuration: &mut Configuration,
        args: &[&str],
    ) -> parser::Result<()> {
        let (command, args) = multiple_args(COMMAND, args, 1)?;
        assert!(command.len() == 1);
        configuration.post_hooks.push(Box::new(PostInstallExecHook {
            current_dir: state.prefix.current.clone(),
            cmd: command[0].to_owned(),
            args: args.iter().cloned().map(String::from).collect(),
        }));
        Ok(())
    }
}
