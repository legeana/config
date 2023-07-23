use std::path::{Path, PathBuf};

use crate::module::{Module, ModuleBox, Rules};
use crate::registry::Registry;

use super::engine;
use super::file_util;
use super::inventory;
use super::util;

use anyhow::{Context, Result};
use indoc::formatdoc;
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
}

impl engine::Statement for SymlinkTreeStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        Ok(Some(Box::new(SymlinkTree {
            src: self.workdir.join(&self.directory),
            dst: ctx.dst_path(&self.directory),
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
    fn parse(&self, workdir: &Path, args: &[&str]) -> Result<engine::StatementBox> {
        let directory = util::single_arg(&self.name(), args)?.to_owned();
        Ok(Box::new(SymlinkTreeStatement {
            workdir: workdir.to_owned(),
            directory,
        }))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(SymlinkTreeBuilder {}));
}
