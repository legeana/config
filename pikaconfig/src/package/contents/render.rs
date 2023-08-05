use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use indoc::formatdoc;

use crate::module::{Module, ModuleBox, Rules};
use crate::registry::Registry;
use crate::tera_helpers;

use super::args::Arguments;
use super::engine;
use super::inventory;
use super::local_state;

const TEMPLATE_NAME: &str = "template";

struct Render {
    tera: tera::Tera,
    context: tera::Context,
    output: local_state::FileState,
}

impl Module for Render {
    fn install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        self.output.install(rules, registry)?;
        let mut file = std::fs::File::create(self.output.path())
            .with_context(|| format!("failed to create a file {:?}", self.output.path()))?;
        self.tera
            .render_to(TEMPLATE_NAME, &self.context, &mut file)
            .with_context(|| format!("failed to render to file {:?}", self.output.path()))?;
        Ok(())
    }
}

#[derive(Debug)]
struct RenderStatement {
    workdir: PathBuf,
    src: String,
    dst: String,
}

impl engine::Statement for RenderStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        let src = self.workdir.join(&self.src);
        let dst = ctx.dst_path(&self.dst);
        let output = local_state::FileState::new(dst.clone())
            .with_context(|| format!("failed to create FileState from {dst:?}"))?;
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
        Ok(Some(Box::new(Render {
            tera,
            context,
            output,
        })))
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
        let filename = args
            .expect_single_arg(self.name())?
            .expect_raw()
            .context("template")?;
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
            src: src.expect_raw().context("template")?.to_owned(),
            dst: dst.expect_raw().context("destination")?.to_owned(),
        }))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(RenderBuilder {}));
    registry.register_command(Box::new(RenderToBuilder {}));
}
