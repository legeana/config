use anyhow::{Context, Result};

use super::local_state;
use crate::package::contents::parser;
use crate::package::contents::util;
use crate::registry::Registry;

pub struct OutputFileParser {}

const COMMAND: &str = "output_file";

struct OutputFile {
    output: local_state::FileState,
}

impl super::Module for OutputFile {
    fn install(&self, rules: &super::Rules, registry: &mut dyn Registry) -> Result<()> {
        self.output.install(rules, registry)
    }
}

impl parser::Parser for OutputFileParser {
    fn name(&self) -> &'static str {
        COMMAND
    }
    fn help(&self) -> &'static str {
        "output_file <filename>
           create a symlink for filename in prefix to a local persistent state"
    }
    fn parse(
        &self,
        state: &mut parser::State,
        configuration: &mut super::Configuration,
        args: &[&str],
    ) -> Result<()> {
        let filename = util::single_arg(COMMAND, args)?;
        let dst = state.prefix.current.join(filename);
        let output = local_state::FileState::new(dst.clone())
            .with_context(|| format!("failed to create FileState for {dst:?}"))?;
        configuration.modules.push(Box::new(OutputFile { output }));
        Ok(())
    }
}
