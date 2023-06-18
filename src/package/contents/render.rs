use std::path::PathBuf;

use anyhow::{Context, Result};
use indoc::formatdoc;
use serde::Serialize;

use crate::module::{Module, Rules};

use super::builder;
use super::local_state;
use super::util;

const TEMPLATE_NAME: &'static str = "template";

#[derive(Serialize)]
struct RenderData {
    workdir: PathBuf,
    prefix: PathBuf,
}

struct Render {
    hb: handlebars::Handlebars<'static>,
    output: local_state::FileState,
    data: RenderData,
}

impl Module for Render {
    fn install(&self, rules: &Rules, registry: &mut dyn crate::registry::Registry) -> Result<()> {
        self.output.install(rules, registry)?;
        let mut file = std::fs::File::create(self.output.path())
            .with_context(|| format!("failed to create a file {:?}", self.output.path()))?;
        self.hb
            .render_to_write(TEMPLATE_NAME, &self.data, &mut file)
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
        let mut hb = handlebars::Handlebars::new();
        hb.register_escape_fn(handlebars::no_escape);
        hb.register_template_file(TEMPLATE_NAME, &src)
            .with_context(|| format!("failed to load template from {src:?}"))?;
        Ok(Some(Box::new(Render {
            hb,
            output,
            data: RenderData {
                workdir: self.workdir.clone(),
                prefix: state.prefix.clone(),
            },
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
