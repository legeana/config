use std::path::PathBuf;
use std::process;

use super::parser;
use super::util;

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
        let cmdline = format!(
            "{:?} $ {} {}",
            self.current_dir,
            shlex::quote(&self.cmd),
            shlex::join(self.args.iter().map(String::as_str))
        );
        log::info!("Executing {cmdline}");
        let status = process::Command::new(&self.cmd)
            .args(&self.args)
            .current_dir(&self.current_dir)
            .status()
            .with_context(|| cmdline.clone())?;
        if !status.success() {
            log::error!("failed to run {cmdline}");
            return Err(anyhow!("failed to execute {cmdline}"));
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
        configuration: &mut super::Configuration,
        args: &[&str],
    ) -> Result<()> {
        let (command, args) = util::multiple_args(COMMAND, args, 1)?;
        assert!(command.len() == 1);
        let args: Vec<String> = args
            .iter()
            .map(shellexpand::tilde)
            .map(String::from)
            .collect();
        configuration.post_hooks.push(Box::new(PostInstallExecHook {
            current_dir: state.prefix.current.clone(),
            cmd: command[0].to_owned(),
            args,
        }));
        Ok(())
    }
}
