use std::path::PathBuf;

use anyhow::Result;

use crate::module::{Module, Rules};
use crate::registry::Registry;

use super::file_util;
use super::parser;
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

impl parser::Builder for SymlinkBuilder {
    fn name(&self) -> &'static str {
        COMMAND
    }
    fn help(&self) -> &'static str {
        "symlink <filename>
           create a symlink for filename in prefix"
    }
    fn build(&self, state: &mut parser::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        let filename = util::single_arg(COMMAND, args)?;
        Ok(Some(Box::new(Symlink {
            src: state.prefix.src_path(filename),
            dst: state.prefix.dst_path(filename),
        })))
    }
}
