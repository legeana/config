use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context as _, Result, anyhow};
use indoc::formatdoc;
use lontra_registry::Registry;
use minijinja::Environment;

use crate::annotated_path::BoxedAnnotatedPath;
use crate::jinja;
use crate::module::{BoxedModule, Module, Rules};

use super::args::{Argument, Arguments};
use super::engine;
use super::inventory;
use super::local_state;

const TEMPLATE_NAME: &str = "template";

struct Render {
    env: Environment<'static>,
    ctx: Arc<jinja::Context>,
    output: BoxedAnnotatedPath,
    permissions: std::fs::Permissions,
}

impl Module for Render {
    // We need to run in post_install to allow jinja functions such as glob to
    // have access to installed symlinks.
    fn post_install(&self, _rules: &Rules, _registry: &mut dyn Registry) -> Result<()> {
        let mut file = std::fs::File::create(self.output.as_path())
            .with_context(|| format!("failed to create a file {:?}", self.output))?;
        self.env
            .get_template(TEMPLATE_NAME)?
            .render_to_write(self.ctx.as_ref(), &mut file)
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
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<BoxedModule>> {
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
        let ctx = Arc::new(jinja::Context {
            source_file: src.clone(),
            source_dir: src
                .parent()
                .ok_or_else(|| anyhow!("failed to get parent of source_file {src:?}"))?
                .to_owned(),
            destination_file: dst.clone(),
            destination_dir: dst
                .parent()
                .ok_or_else(|| anyhow!("failed to get parent of destination_file {dst:?}"))?
                .to_owned(),
            workdir: self.workdir.clone(),
            prefix: ctx.prefix.clone(),
        });
        inventory::register_render_globals(&mut env, &ctx);
        jinja::register(&mut env, &ctx);
        Ok(Some(Box::new((
            output,
            Render {
                env,
                ctx,
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
