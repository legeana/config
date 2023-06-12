use std::path::PathBuf;

use anyhow::Result;
use indoc::formatdoc;

use crate::module::{Module, Rules};
use crate::registry::Registry;

use super::builder;
use super::file_util;
use super::util;

struct SymlinkBuilder;
struct SymlinkToBuilder;

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
        "symlink".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <filename>
                create a symlink for filename in prefix
        ", command=self.name()}
    }
    fn build(&self, state: &mut builder::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        let filename = util::single_arg(&self.name(), args)?;
        Ok(Some(Box::new(Symlink {
            src: state.prefix.src_path(filename),
            dst: state.prefix.dst_path(filename),
        })))
    }
}

impl builder::Builder for SymlinkToBuilder {
    fn name(&self) -> String {
        "symlink_to".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <destination> <filename>
                create a symlink for filename in prefix
        ", command=self.name()}
    }
    fn build(&self, state: &mut builder::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        let (dst, src) = util::double_arg(&self.name(), args)?;
        Ok(Some(Box::new(Symlink {
            src: state.prefix.src_path(src),
            dst: state.prefix.dst_path(dst),
        })))
    }
}

pub fn commands() -> Vec<Box<dyn builder::Builder>> {
    vec![Box::new(SymlinkBuilder {}), Box::new(SymlinkToBuilder {})]
}
