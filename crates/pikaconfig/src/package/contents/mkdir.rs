use std::path::{Path, PathBuf};

use crate::module::{Module, ModuleBox, Rules};

use super::args::{Argument, Arguments};
use super::engine;
use super::inventory;

use anyhow::{Context as _, Result};
use indoc::formatdoc;
use registry::{FilePath, Registry};

struct MkDir {
    dst: PathBuf,
}

impl Module for MkDir {
    fn install(&self, _rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        std::fs::create_dir_all(&self.dst)
            .with_context(|| format!("unable to create {:?}", self.dst))?;
        registry
            .register_user_file(FilePath::Directory(&self.dst))
            .with_context(|| format!("failed to register directory {:?}", self.dst))?;
        Ok(())
    }
}

#[derive(Debug)]
struct MkDirStatement {
    dir: Argument,
}

impl engine::Statement for MkDirStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        Ok(Some(Box::new(MkDir {
            dst: ctx.dst_path(ctx.expand_arg(&self.dir)?),
        })))
    }
}

#[derive(Clone)]
struct MkDirBuilder;

impl engine::CommandBuilder for MkDirBuilder {
    fn name(&self) -> String {
        "mkdir".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <directory>
                create a directory in prefix
        ", command=self.name()}
    }
    fn build(&self, _workdir: &Path, args: &Arguments) -> Result<engine::Command> {
        let dir = args.expect_single_arg(self.name())?.clone();
        Ok(engine::Command::new_statement(MkDirStatement { dir }))
    }
}

pub(super) fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(MkDirBuilder));
}
