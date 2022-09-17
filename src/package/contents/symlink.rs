use std::path::PathBuf;

use super::file_util;
use super::parser;
use super::util;
use crate::registry::Registry;

pub struct SymlinkParser {}

const COMMAND: &str = "symlink";

struct SymlinkInstaller {
    src: PathBuf,
    dst: PathBuf,
}

impl super::FileInstaller for SymlinkInstaller {
    fn install(&self, registry: &mut dyn Registry) -> anyhow::Result<()> {
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
    ) -> parser::Result<()> {
        let filename = util::single_arg(COMMAND, args)?;
        configuration.files.push(Box::new(SymlinkInstaller {
            src: configuration.root.join(filename),
            dst: state.prefix.current.join(filename),
        }));
        Ok(())
    }
}
