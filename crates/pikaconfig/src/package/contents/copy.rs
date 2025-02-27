use std::path::{Path, PathBuf};

use crate::annotated_path::AnnotatedPathBox;
use crate::module::{Module, ModuleBox, Rules};

use super::args::{Argument, Arguments};
use super::engine;
use super::inventory;
use super::local_state;

use anyhow::{Context as _, Result, anyhow};
use indoc::formatdoc;
use registry::Registry;

struct Copy {
    src: PathBuf,
    output: AnnotatedPathBox,
}

impl Module for Copy {
    fn install(&self, _rules: &Rules, _registry: &mut dyn Registry) -> Result<()> {
        if self
            .output
            .as_path()
            .try_exists()
            .with_context(|| format!("unable to check if {:?} exists", self.output))?
        {
            log::info!(
                "Copy: skipping already existing state for {:?}",
                self.output
            );
            return Ok(());
        }
        std::fs::copy(&self.src, self.output.as_path())
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

#[derive(Debug)]
struct CopyFromStatement {
    path: Argument,
}

impl engine::Statement for CopyFromStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        let src: PathBuf = ctx.expand_arg(&self.path)?.into();
        let dst = ctx.dst_path(
            src.file_name()
                .ok_or_else(|| anyhow!("failed to get basename from {src:?}"))?,
        );
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

#[derive(Clone)]
struct CopyFromBuilder;

impl engine::CommandBuilder for CopyFromBuilder {
    fn name(&self) -> String {
        "copy_from".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <path>
                create a copy of <path> to <path.basename> in prefix
        ", command=self.name()}
    }
    fn build(&self, _workdir: &Path, args: &Arguments) -> Result<engine::Command> {
        let path = args.expect_single_arg(self.name())?.clone();
        Ok(engine::Command::new_statement(CopyFromStatement { path }))
    }
}

#[derive(Clone)]
struct CopyToBuilder;

impl engine::CommandBuilder for CopyToBuilder {
    fn name(&self) -> String {
        "copy_to".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <destination> <filename>
                create a copy of a filename in local storage and install a symlink to it
        ", command=self.name()}
    }
    fn build(&self, workdir: &Path, args: &Arguments) -> Result<engine::Command> {
        let (dst, src) = args.expect_double_arg(self.name())?;
        Ok(engine::Command::new_statement(CopyStatement {
            workdir: workdir.to_owned(),
            src: src.clone(),
            dst: dst.clone(),
        }))
    }
}

pub(super) fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(CopyBuilder));
    registry.register_command(Box::new(CopyFromBuilder));
    registry.register_command(Box::new(CopyToBuilder));
}
