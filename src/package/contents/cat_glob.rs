use std::io::Write;
use std::path::Path;

use crate::module::{Module, ModuleBox, Rules};
use crate::registry::Registry;

use super::ast;
use super::inventory;
use super::local_state;
use super::util;

use anyhow::{anyhow, Context, Result};
use glob::glob as glob_iter;
use indoc::formatdoc;

struct CatGlobInto {
    globs: Vec<String>,
    output: local_state::FileState,
}

impl Module for CatGlobInto {
    fn install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        self.output.install(rules, registry)
    }
    fn post_install(&self, _rules: &super::Rules, _registry: &mut dyn Registry) -> Result<()> {
        let out_file = std::fs::File::create(self.output.path())
            .with_context(|| format!("unable to create {:?}", self.output.path()))?;
        let mut out = std::io::BufWriter::new(out_file);
        for glob in self.globs.iter() {
            for entry in glob_iter(glob).with_context(|| format!("failed to glob {}", glob))? {
                let path =
                    entry.with_context(|| format!("failed to iterate over glob {}", glob))?;
                let inp_file = std::fs::File::open(&path)
                    .with_context(|| format!("failed to open {path:?}"))?;
                let mut inp = std::io::BufReader::new(inp_file);
                std::io::copy(&mut inp, &mut out).with_context(|| {
                    format!("failed to copy from {path:?} to {:?}", self.output.path())
                })?;
            }
        }
        out.flush()
            .with_context(|| format!("failed to flush {:?}", self.output.path()))?;
        Ok(())
    }
}

#[derive(Debug)]
struct CatGlobIntoStatement {
    filename: String,
    globs: Vec<String>,
}

impl ast::Statement for CatGlobIntoStatement {
    fn eval(&self, state: &mut ast::State) -> Result<Option<ModuleBox>> {
        let current_prefix = state.prefix.to_str().ok_or_else(|| {
            anyhow!(
                "failed to represent current prefix {:?} as a string",
                &state.prefix
            )
        })?;
        let glob_prefix = current_prefix.to_owned() + std::path::MAIN_SEPARATOR_STR;
        let concatenated_globs: Vec<String> =
            self.globs.iter().map(|g| glob_prefix.clone() + g).collect();
        let dst = state.dst_path(&self.filename);
        let output = local_state::FileState::new(dst.clone())
            .with_context(|| format!("failed to create FileState for {dst:?}"))?;
        Ok(Some(Box::new(CatGlobInto {
            globs: concatenated_globs,
            output,
        })))
    }
}

#[derive(Clone)]
struct CatGlobIntoParser;

impl ast::Parser for CatGlobIntoParser {
    fn name(&self) -> String {
        "cat_glob_into".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <filename> <glob1> [<glob2> ...]
                create filename in local storage by concatenating globs
        ", command=self.name()}
    }
    fn parse(&self, _workdir: &Path, args: &[&str]) -> Result<ast::StatementBox> {
        let (fname, globs) = util::multiple_args(&self.name(), args, 1)?;
        assert!(fname.len() == 1);
        let filename = fname[0].to_owned();
        let globs: Vec<_> = globs
            .iter()
            .map(|&s| s.to_owned())
            //.map(String::from)
            .collect();
        Ok(Box::new(CatGlobIntoStatement { filename, globs }))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_parser(Box::new(CatGlobIntoParser {}));
}
