use std::path::PathBuf;

use anyhow::Result;
use indoc::formatdoc;

use crate::module::{Module, Rules};
use crate::registry::Registry;

use super::builder;
use super::file_util;
use super::util;

pub struct SymlinkBuilder;

const COMMAND: &str = "symlink";

struct Symlink {
    src: PathBuf,
    dst: PathBuf,
}

impl Module for Symlink {
    fn install(&self, _rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        file_util::make_symlink(registry, &self.src, &self.dst)
    }
}

impl builder::Builder for SymlinkBuilder {
    fn name(&self) -> String {
        COMMAND.to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {COMMAND} <filename>
                create a symlink for filename in prefix
        "}
    }
    fn build(&self, state: &mut builder::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        let filename = util::single_arg(COMMAND, args)?;
        Ok(Some(Box::new(Symlink {
            src: state.prefix.src_path(filename),
            dst: state.prefix.dst_path(filename),
        })))
    }
}
