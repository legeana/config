use std::io::Write as _;
use std::path::{Path, PathBuf};

use crate::annotated_path::BoxedAnnotatedPath;
use crate::module::{BoxedModule, Module, Rules};

use super::args::{Argument, Arguments};
use super::engine;
use super::inventory;
use super::local_state;

use anyhow::{Context as _, Result};
use indoc::formatdoc;
use lontra_glob::glob_iter;
use lontra_registry::Registry;

struct CatGlobInto {
    glob_root: PathBuf,
    globs: Vec<String>,
    output: BoxedAnnotatedPath,
}

impl Module for CatGlobInto {
    fn post_install(&self, _rules: &Rules, _registry: &mut dyn Registry) -> Result<()> {
        let out_file = std::fs::File::create(self.output.as_path())
            .with_context(|| format!("unable to create {:?}", self.output))?;
        let mut out = std::io::BufWriter::new(out_file);
        for glob in &self.globs {
            for entry in glob_iter(&self.glob_root, glob)
                .with_context(|| format!("failed to glob {glob:?}"))?
            {
                let path = entry.with_context(|| format!("failed to iterate over glob {glob}"))?;
                let inp_file = std::fs::File::open(&path)
                    .with_context(|| format!("failed to open {path:?}"))?;
                let mut inp = std::io::BufReader::new(inp_file);
                std::io::copy(&mut inp, &mut out).with_context(|| {
                    format!("failed to copy from {path:?} to {:?}", self.output)
                })?;
            }
        }
        out.flush()
            .with_context(|| format!("failed to flush {:?}", self.output))?;
        Ok(())
    }
}

#[derive(Debug)]
struct CatGlobIntoStatement {
    filename: Argument,
    globs: Vec<String>,
}

impl engine::Statement for CatGlobIntoStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<BoxedModule>> {
        let dst = ctx.dst_path(ctx.expand_arg(&self.filename)?);
        let output = local_state::file_state(dst.clone())
            .with_context(|| format!("failed to create FileState for {dst:?}"))?;
        let output_state = output.state();
        Ok(Some(Box::new((
            output,
            CatGlobInto {
                glob_root: ctx.prefix.clone(),
                globs: self.globs.clone(),
                output: output_state,
            },
        ))))
    }
}

#[derive(Clone)]
struct CatGlobIntoBuilder;

impl engine::CommandBuilder for CatGlobIntoBuilder {
    fn name(&self) -> String {
        "cat_glob_into".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <filename> <glob1> [<glob2> ...]
                create filename in local storage by concatenating globs
        ", command=self.name()}
    }
    fn build(&self, _workdir: &Path, args: &Arguments) -> Result<engine::Command> {
        let (filename, globs) = args.expect_at_least_one_arg(self.name())?;
        let filename = filename.clone();
        let globs: Vec<_> = globs
            .iter()
            .map(|g| g.expect_raw().context("glob").map(str::to_string))
            .collect::<Result<_>>()?;
        Ok(engine::Command::new_statement(CatGlobIntoStatement {
            filename,
            globs,
        }))
    }
}

pub(super) fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(CatGlobIntoBuilder));
}
