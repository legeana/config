use std::path::PathBuf;

use anyhow::{Context, Result};
use indoc::formatdoc;

use crate::module::{Module, Rules};
use crate::tera_helpers;

use super::builder;
use super::local_state;
use super::util;

const TEMPLATE_NAME: &str = "template";

struct Render {
    tera: tera::Tera,
    context: tera::Context,
    output: local_state::FileState,
}

impl Module for Render {
    fn install(&self, rules: &Rules, registry: &mut dyn crate::registry::Registry) -> Result<()> {
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
struct RenderBuilder {
    workdir: PathBuf,
    filename: String,
}

impl builder::Builder for RenderBuilder {
    fn build(&self, state: &mut builder::State) -> Result<Option<Box<dyn Module>>> {
        let src = self.workdir.join(&self.filename);
        let dst = state.dst_path(&self.filename);
        let output = local_state::FileState::new(dst.clone())
            .with_context(|| format!("failed to create FileState from {dst:?}"))?;
        let mut tera = tera::Tera::default();
        tera.add_template_file(&src, Some(TEMPLATE_NAME))
            .with_context(|| format!("failed to load template from {src:?}"))?;
        builder::register_render_helpers(&mut tera)?;
        tera_helpers::register(&mut tera)?;
        let mut context = tera::Context::new();
        context.insert("source_file", &src);
        context.insert("destination_file", &dst);
        context.insert("workdir", &self.workdir);
        context.insert("prefix", &state.prefix);
        Ok(Some(Box::new(Render {
            tera,
            context,
            output,
        })))
    }
}

#[derive(Clone)]
struct RenderParser;

impl builder::Parser for RenderParser {
    fn name(&self) -> String {
        "render".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <filename>
                render template
        ", command=self.name()}
    }
    fn parse(
        &self,
        workdir: &std::path::Path,
        args: &[&str],
    ) -> anyhow::Result<Box<dyn builder::Builder>> {
        let filename = util::single_arg(&self.name(), args)?.to_owned();
        Ok(Box::new(RenderBuilder {
            workdir: workdir.to_owned(),
            filename,
        }))
    }
}

pub fn commands() -> Vec<Box<dyn builder::Parser>> {
    vec![Box::new(RenderParser {})]
}
