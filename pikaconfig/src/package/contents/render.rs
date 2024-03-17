use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use indoc::formatdoc;

use crate::module::{Module, ModuleBox, Rules};
use crate::registry::Registry;
use crate::tera_helpers;

use super::args::{Argument, Arguments};
use super::engine;
use super::inventory;
use super::local_state;

const TEMPLATE_NAME: &str = "template";

struct Render {
    tera: tera::Tera,
    context: tera::Context,
    output: local_state::StateBox,
}

impl Module for Render {
    fn install(&self, _rules: &Rules, _registry: &mut dyn Registry) -> Result<()> {
        let mut file = std::fs::File::create(self.output.path())
            .with_context(|| format!("failed to create a file {:?}", self.output))?;
        self.tera
            .render_to(TEMPLATE_NAME, &self.context, &mut file)
            .with_context(|| format!("failed to render to file {:?}", self.output))?;
        file.sync_all()
            .with_context(|| format!("failed to flush {:?}", self.output))
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
        let mut tera = tera::Tera::default();
        tera.add_template_file(&src, Some(TEMPLATE_NAME))
            .with_context(|| format!("failed to load template from {src:?}"))?;
        inventory::register_render_helpers(&mut tera);
        tera_helpers::register(&mut tera);
        let mut context = tera::Context::new();
        context.insert("source_file", &src);
        context.insert("destination_file", &dst);
        context.insert("workdir", &self.workdir);
        context.insert("prefix", &ctx.prefix);
        Ok(Some(Box::new((
            output,
            Render {
                tera,
                context,
                output: output_state,
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
    fn build(&self, workdir: &Path, args: &Arguments) -> anyhow::Result<engine::Command> {
        let filename = args.expect_single_arg(self.name())?.clone();
        Ok(engine::Command::new_statement(RenderStatement {
            workdir: workdir.to_owned(),
            src: filename.to_owned(),
            dst: filename.to_owned(),
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

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(RenderBuilder));
    registry.register_command(Box::new(RenderToBuilder));
}
