use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use indoc::formatdoc;
use minijinja::Environment;
use registry::Registry;
use serde::Serialize;

use crate::annotated_path::AnnotatedPathBox;
use crate::minijinja_helper;
use crate::module::{Module, ModuleBox, Rules};

use super::args::{Argument, Arguments};
use super::engine;
use super::inventory;
use super::local_state;

const TEMPLATE_NAME: &str = "template";

#[derive(Debug, Serialize)]
struct Context {
    source_file: PathBuf,
    destination_file: PathBuf,
    workdir: PathBuf,
    prefix: PathBuf,
}

struct Render {
    env: Environment<'static>,
    ctx: Context,
    output: AnnotatedPathBox,
    permissions: std::fs::Permissions,
}

impl Module for Render {
    fn install(&self, _rules: &Rules, _registry: &mut dyn Registry) -> Result<()> {
        let mut file = std::fs::File::create(self.output.as_path())
            .with_context(|| format!("failed to create a file {:?}", self.output))?;
        self.env
            .get_template(TEMPLATE_NAME)?
            .render_to_write(&self.ctx, &mut file)
            .with_context(|| format!("failed to render to file {:?}", self.output))?;
        file.sync_all()
            .with_context(|| format!("failed to flush {:?}", self.output))?;
        file.set_permissions(self.permissions.clone())
            .with_context(|| format!("failed to set permissions to file {:?}", self.output))
    }
}

#[derive(Debug)]
struct RenderStatement {
    workdir: PathBuf,
    src: Argument,
    dst: Argument,
}

impl engine::Statement for RenderStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        let src = self.workdir.join(ctx.expand_arg(&self.src)?);
        let dst = ctx.dst_path(ctx.expand_arg(&self.dst)?);
        let output = local_state::file_state(dst.clone())
            .with_context(|| format!("failed to create FileState from {dst:?}"))?;
        let output_state = output.state();
        let mut env = Environment::new();
        let tmpl_data = std::fs::read_to_string(&src)
            .with_context(|| format!("failed to read template from {src:?}"))?;
        env.add_template_owned(TEMPLATE_NAME, tmpl_data)
            .with_context(|| format!("failed to load template from {src:?}"))?;
        let permissions = std::fs::metadata(&src)
            .with_context(|| format!("failed to load {src:?} metadata"))?
            .permissions();
        inventory::register_render_helpers(&mut env);
        minijinja_helper::register(&mut env);
        Ok(Some(Box::new((
            output,
            Render {
                env,
                ctx: Context {
                    source_file: src.clone(),
                    destination_file: dst.clone(),
                    workdir: self.workdir.clone(),
                    prefix: ctx.prefix.clone(),
                },
                output: output_state,
                permissions,
            },
        ))))
    }
}

#[derive(Clone)]
struct RenderBuilder;

impl engine::CommandBuilder for RenderBuilder {
    fn name(&self) -> String {
        "render".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <filename>
                render template
        ", command=self.name()}
    }
    fn build(&self, workdir: &Path, args: &Arguments) -> Result<engine::Command> {
        let filename = args.expect_single_arg(self.name())?.clone();
        Ok(engine::Command::new_statement(RenderStatement {
            workdir: workdir.to_owned(),
            src: filename.clone(),
            dst: filename,
        }))
    }
}

#[derive(Clone)]
struct RenderToBuilder;

impl engine::CommandBuilder for RenderToBuilder {
    fn name(&self) -> String {
        "render_to".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <destination> <filename>
                render template <filename> into <destination>
        ", command=self.name()}
    }
    fn build(&self, workdir: &Path, args: &Arguments) -> Result<engine::Command> {
        let (dst, src) = args.expect_double_arg(self.name())?;
        Ok(engine::Command::new_statement(RenderStatement {
            workdir: workdir.to_owned(),
            src: src.clone(),
            dst: dst.clone(),
        }))
    }
}

pub(super) fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(RenderBuilder));
    registry.register_command(Box::new(RenderToBuilder));
}
