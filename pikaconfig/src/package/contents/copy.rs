use std::path::{Path, PathBuf};

use crate::module::{Module, ModuleBox, Rules};
use crate::registry::Registry;

use super::args::{Argument, Arguments};
use super::engine;
use super::inventory;
use super::local_state;

use anyhow::{Context, Result};
use indoc::formatdoc;

struct Copy {
    src: PathBuf,
    output: local_state::StateBox,
}

impl Module for Copy {
    fn install(&self, _rules: &Rules, _registry: &mut dyn Registry) -> Result<()> {
        if self
            .output
            .path()
            .try_exists()
            .with_context(|| format!("unable to check if {:?} exists", self.output))?
        {
            log::info!(
                "Copy: skipping already existing state for {:?}",
                self.output
            );
            return Ok(());
        }
        std::fs::copy(&self.src, self.output.path())
            .with_context(|| format!("unable to copy {:?} to {:?}", self.src, self.output))?;
        Ok(())
    }
}

#[derive(Debug)]
struct CopyStatement {
    workdir: PathBuf,
    src: Argument,
    dst: Argument,
}

impl engine::Statement for CopyStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        let src = self.workdir.join(ctx.expand_arg(&self.src)?);
        let dst = ctx.dst_path(ctx.expand_arg(&self.dst)?);
        let output = local_state::file_state(dst.clone())
            .with_context(|| format!("failed to create FileState from {dst:?}"))?;
        let output_state = output.state();
        Ok(Some(Box::new((
            output,
            Copy {
                src,
                output: output_state,
            },
        ))))
    }
}

#[derive(Clone)]
struct CopyBuilder;

impl engine::CommandBuilder for CopyBuilder {
    fn name(&self) -> String {
        "copy".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <filename>
                create a copy of a filename in local storage and install a symlink to it
        ", command=self.name()}
    }
    fn build(&self, workdir: &Path, args: &Arguments) -> Result<engine::Command> {
        let filename = args.expect_single_arg(self.name())?;
        Ok(engine::Command::new_statement(CopyStatement {
            workdir: workdir.to_owned(),
            src: filename.clone(),
            dst: filename.clone(),
        }))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(CopyBuilder {}));
}
