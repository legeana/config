use std::path::PathBuf;

use anyhow::Result;

use crate::package::contents::file_util;
use crate::package::contents::parser;
use crate::package::contents::util;
use crate::registry::Registry;

pub struct OutputFileParser {}

const COMMAND: &str = "output_file";

struct OutputFile {
    dst: PathBuf,
}

impl super::Module for OutputFile {
    fn install(&self, registry: &mut dyn Registry) -> Result<()> {
        file_util::make_local_state(registry, &self.dst).map(|_| ())
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
        configuration.modules.push(Box::new(OutputFile {
            dst: state.prefix.current.join(filename),
        }));
        Ok(())
    }
}
