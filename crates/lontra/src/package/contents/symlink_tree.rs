use std::path::{Path, PathBuf};

use crate::module::{BoxedModule, Module, Rules};

use super::args::Arguments;
use super::engine;
use super::file_util;
use super::inventory;

use anyhow::{Context as _, Result};
use indoc::formatdoc;
use lontra_registry::Registry;
use walkdir::WalkDir;

struct SymlinkTree {
    src: PathBuf,
    dst: PathBuf,
}

impl Module for SymlinkTree {
    fn install(&self, _rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        for e in WalkDir::new(&self.src).sort_by_file_name() {
            let entry = e.with_context(|| format!("failed to read {:?}", self.src))?;
            if entry.file_type().is_dir() {
                // Skip directories. They will be automatically created by make_symlink().
                continue;
            }
            let src = entry.path();
            let suffix = src
                .strip_prefix(&self.src)
                .with_context(|| format!("unable to remove prefix {:?} from {src:?}", self.src))?;
            let dst = self.dst.join(suffix);
            file_util::make_symlink(registry, src, &dst)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct SymlinkTreeStatement {
    workdir: PathBuf,
    directory: String,
    destination: String,
}

impl engine::Statement for SymlinkTreeStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<BoxedModule>> {
        Ok(Some(Box::new(SymlinkTree {
            src: self.workdir.join(&self.directory),
            dst: ctx.dst_path(&self.destination),
        })))
    }
}

#[derive(Clone)]
struct SymlinkTreeBuilder;

impl engine::CommandBuilder for SymlinkTreeBuilder {
    fn name(&self) -> String {
        "symlink_tree".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <directory>
                create a symlink for every file in a directory recursively
        ", command=self.name()}
    }
    fn build(&self, workdir: &Path, args: &Arguments) -> Result<engine::Command> {
        let directory = args
            .expect_single_arg(self.name())?
            .expect_raw()
            .context("directory")?
            .to_owned();
        let destination = directory.clone();
        Ok(engine::Command::new_statement(SymlinkTreeStatement {
            workdir: workdir.to_owned(),
            directory,
            destination,
        }))
    }
}

#[derive(Clone)]
struct SymlinkTreeToBuilder;

impl engine::CommandBuilder for SymlinkTreeToBuilder {
    fn name(&self) -> String {
        "symlink_tree_to".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <destination> <directory>
                create a symlink for every file in a directory recursively
                into the destination directory:
                    <directory>/foo/bar -> <destination>/foo/bar
        ", command=self.name()}
    }
    fn build(&self, workdir: &Path, args: &Arguments) -> Result<engine::Command> {
        let (destination, directory) = args.expect_double_arg(self.name())?;
        let destination = destination.expect_raw().context("destination")?.to_owned();
        let directory = directory.expect_raw().context("directory")?.to_owned();
        Ok(engine::Command::new_statement(SymlinkTreeStatement {
            workdir: workdir.to_owned(),
            directory,
            destination,
        }))
    }
}

pub(super) fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(SymlinkTreeBuilder));
    registry.register_command(Box::new(SymlinkTreeToBuilder));
}
