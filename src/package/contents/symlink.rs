use std::path::{Path, PathBuf};

use anyhow::Result;
use indoc::formatdoc;

use crate::module::{Module, ModuleBox, Rules};
use crate::registry::Registry;

use super::ast;
use super::engine;
use super::file_util;
use super::inventory;
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
struct SymlinkStatement {
    workdir: PathBuf,
    src: String,
    dst: String,
}

impl ast::Statement for SymlinkStatement {
    fn eval(&self, state: &mut engine::State) -> Result<Option<ModuleBox>> {
        Ok(Some(Box::new(Symlink {
            src: self.workdir.join(&self.src),
            dst: state.dst_path(&self.dst),
        })))
    }
}

#[derive(Clone)]
struct SymlinkParser;

impl ast::Parser for SymlinkParser {
    fn name(&self) -> String {
        "symlink".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <filename>
                create a symlink for filename in prefix
        ", command=self.name()}
    }
    fn parse(&self, workdir: &Path, args: &[&str]) -> Result<ast::StatementBox> {
        let filename = util::single_arg(&self.name(), args)?;
        Ok(Box::new(SymlinkStatement {
            workdir: workdir.to_owned(),
            src: filename.to_owned(),
            dst: filename.to_owned(),
        }))
    }
}

#[derive(Clone)]
struct SymlinkToParser;

impl ast::Parser for SymlinkToParser {
    fn name(&self) -> String {
        "symlink_to".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <destination> <filename>
                create a symlink for filename in prefix
        ", command=self.name()}
    }
    fn parse(&self, workdir: &Path, args: &[&str]) -> Result<ast::StatementBox> {
        let (dst, src) = util::double_arg(&self.name(), args)?;
        Ok(Box::new(SymlinkStatement {
            workdir: workdir.to_owned(),
            src: src.to_owned(),
            dst: dst.to_owned(),
        }))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_parser(Box::new(SymlinkParser {}));
    registry.register_parser(Box::new(SymlinkToParser {}));
}
