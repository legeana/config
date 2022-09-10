use std::path::PathBuf;

use crate::package::configuration::file_util::make_local_state;
use crate::package::configuration::parser;
use crate::package::configuration::util::single_arg;
use crate::package::configuration::Configuration;
use crate::registry::Registry;

use anyhow;

pub struct OutputFileParser {}

const COMMAND: &str = "output_file";

struct OutputFileInstaller {
    dst: PathBuf,
}

impl super::FileInstaller for OutputFileInstaller {
    fn install(&self, registry: &mut dyn Registry) -> anyhow::Result<()> {
        make_local_state(registry, &self.dst).map(|_| ())
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
        configuration: &mut Configuration,
        args: &[&str],
    ) -> parser::Result<()> {
        let filename = single_arg(COMMAND, args)?;
        configuration.files.push(Box::new(OutputFileInstaller {
            dst: state.prefix.current.join(filename),
        }));
        return Ok(());
    }
}
