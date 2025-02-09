use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use indoc::formatdoc;

use crate::module::{Module, ModuleBox, Rules};
use crate::registry::Registry;

use super::args::{Argument, Arguments};
use super::engine;
use super::file_util;
use super::inventory;

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
    src: Argument,
    dst: Argument,
}

impl engine::Statement for SymlinkStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        let src = ctx.expand_arg(&self.src)?;
        let dst = ctx.expand_arg(&self.dst)?;
        Ok(Some(Box::new(Symlink {
            src: self.workdir.join(src),
            dst: ctx.dst_path(dst),
        })))
    }
}

#[derive(Debug)]
struct SymlinkFromStatement {
    path: Argument,
}

impl engine::Statement for SymlinkFromStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        let src: PathBuf = ctx.expand_arg(&self.path)?.into();
        let dst = ctx.dst_path(
            src.file_name()
                .ok_or_else(|| anyhow!("failed to get basename from {src:?}"))?,
        );
        Ok(Some(Box::new(Symlink { src, dst })))
    }
}

#[derive(Clone)]
struct SymlinkBuilder;

impl engine::CommandBuilder for SymlinkBuilder {
    fn name(&self) -> String {
        "symlink".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <filename>
                create a symlink for filename in prefix
        ", command=self.name()}
    }
    fn build(&self, workdir: &Path, args: &Arguments) -> Result<engine::Command> {
        let filename = args.expect_single_arg(self.name())?;
        Ok(engine::Command::new_statement(SymlinkStatement {
            workdir: workdir.to_owned(),
            src: filename.clone(),
            dst: filename.clone(),
        }))
    }
}

#[derive(Clone)]
struct SymlinkToBuilder;

impl engine::CommandBuilder for SymlinkToBuilder {
    fn name(&self) -> String {
        "symlink_to".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <destination> <filename>
                create a symlink for filename in prefix
        ", command=self.name()}
    }
    fn build(&self, workdir: &Path, args: &Arguments) -> Result<engine::Command> {
        let (dst, src) = args.expect_double_arg(self.name())?;
        Ok(engine::Command::new_statement(SymlinkStatement {
            workdir: workdir.to_owned(),
            src: src.clone(),
            dst: dst.clone(),
        }))
    }
}

#[derive(Clone)]
struct SymlinkFromBuilder;

impl engine::CommandBuilder for SymlinkFromBuilder {
    fn name(&self) -> String {
        "symlink_from".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <path>
                create a symlink to <path> from <path.basename> in prefix
        ", command=self.name()}
    }
    fn build(&self, _workdir: &Path, args: &Arguments) -> Result<engine::Command> {
        let path = args.expect_single_arg(self.name())?.clone();
        Ok(engine::Command::new_statement(SymlinkFromStatement {
            path,
        }))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(SymlinkBuilder));
    registry.register_command(Box::new(SymlinkToBuilder));
    registry.register_command(Box::new(SymlinkFromBuilder));
}
