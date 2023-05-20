use std::path::PathBuf;

use anyhow::Result;

use super::file_util;
use super::parser;
use super::util;
use crate::registry::Registry;

pub struct SymlinkParser {}

const COMMAND: &str = "symlink";

struct Symlink {
    src: PathBuf,
    dst: PathBuf,
}

impl super::Module for Symlink {
    fn install(&self, registry: &mut dyn Registry) -> Result<()> {
        file_util::make_symlink(registry, &self.src, &self.dst)
    }
}

impl parser::Parser for SymlinkParser {
    fn name(&self) -> &'static str {
        COMMAND
    }
    fn help(&self) -> &'static str {
        "symlink <filename>
           create a symlink for filename in prefix"
    }
    fn parse(
        &self,
        state: &mut parser::State,
        configuration: &mut super::Configuration,
        args: &[&str],
    ) -> Result<()> {
        let filename = util::single_arg(COMMAND, args)?;
        configuration.modules.push(Box::new(Symlink {
            src: configuration.root.join(filename),
            dst: state.prefix.current.join(filename),
        }));
        Ok(())
    }
}
