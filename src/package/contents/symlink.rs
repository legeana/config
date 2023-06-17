use std::path::{Path, PathBuf};

use anyhow::Result;
use indoc::formatdoc;

use crate::module::{Module, Rules};
use crate::registry::Registry;

use super::builder;
use super::file_util;
use super::util;

struct Symlink {
    src: PathBuf,
    dst: PathBuf,
}

impl Module for Symlink {
    fn install(&self, _rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        file_util::make_symlink(registry, &self.src, &self.dst)
    }
}

#[derive(Debug)]
struct SymlinkBuilder {
    workdir: PathBuf,
    src: String,
    dst: String,
}

impl builder::Builder for SymlinkBuilder {
    fn build(&self, state: &mut builder::State) -> Result<Option<Box<dyn Module>>> {
        Ok(Some(Box::new(Symlink {
            src: self.workdir.join(&self.src),
            dst: state.dst_path(&self.dst),
        })))
    }
}

#[derive(Clone)]
struct SymlinkParser;

impl builder::Parser for SymlinkParser {
    fn name(&self) -> String {
        "symlink".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <filename>
                create a symlink for filename in prefix
        ", command=self.name()}
    }
    fn parse(&self, workdir: &Path, args: &[&str]) -> Result<Box<dyn builder::Builder>> {
        let filename = util::single_arg(&self.name(), args)?;
        Ok(Box::new(SymlinkBuilder {
            workdir: workdir.to_owned(),
            src: filename.to_owned(),
            dst: filename.to_owned(),
        }))
    }
}

#[derive(Clone)]
struct SymlinkToParser;

impl builder::Parser for SymlinkToParser {
    fn name(&self) -> String {
        "symlink_to".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <destination> <filename>
                create a symlink for filename in prefix
        ", command=self.name()}
    }
    fn parse(&self, workdir: &Path, args: &[&str]) -> Result<Box<dyn builder::Builder>> {
        let (dst, src) = util::double_arg(&self.name(), args)?;
        Ok(Box::new(SymlinkBuilder {
            workdir: workdir.to_owned(),
            src: src.to_owned(),
            dst: dst.to_owned(),
        }))
    }
}

pub fn commands() -> Vec<Box<dyn builder::Parser>> {
    vec![Box::new(SymlinkParser {}), Box::new(SymlinkToParser {})]
}
