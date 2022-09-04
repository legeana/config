use std::path::PathBuf;

use crate::package::configuration::file_util::make_symlink;
use crate::package::configuration::parser;
use crate::package::configuration::util::single_arg;
use crate::package::configuration::Configuration;
use crate::registry::Registry;

use anyhow;

pub struct SymlinkParser {}

const COMMAND: &str = "symlink";

struct SymlinkInstaller {
    src: PathBuf,
    dst: PathBuf,
}

impl super::FileInstaller for SymlinkInstaller {
    fn install(&self, registry: &mut dyn Registry) -> anyhow::Result<()> {
        make_symlink(&self.src, &self.dst)?;
        registry.register_symlink(&self.dst)?;
        return Ok(());
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
        configuration: &mut Configuration,
        args: &[&str],
    ) -> parser::Result<()> {
        let filename = single_arg(COMMAND, args)?;
        configuration.files.push(Box::new(SymlinkInstaller {
            src: configuration.root.join(filename),
            dst: state.prefix.current.join(filename),
        }));
        return Ok(());
    }
}
