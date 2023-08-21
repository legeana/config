mod args;
mod ast;
mod cat_glob;
mod copy;
mod deprecated;
mod dirs_prefix;
mod engine;
mod exec;
mod fetch;
mod file_tests;
mod file_util;
mod git_clone;
mod importer;
mod inventory;
mod is_command;
mod is_os;
mod lexer;
mod literal;
mod local_state;
mod mkdir;
mod net_util;
mod once;
mod output_file;
mod parser;
mod prefix;
mod remote_source;
mod render;
mod return_;
mod set_contents;
mod subdir;
mod symlink;
mod symlink_tree;
mod tags;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use lalrpop_util::lalrpop_mod;

use crate::module::{self, ModuleBox};
use crate::package::contents::engine::{Statement, StatementBox};

lalrpop_mod!(ast_parser, "/package/contents/ast_parser.rs");

const MANIFEST: &str = "MANIFEST";

pub use engine::help;

fn error_context(root: &Path) -> String {
    format!("{root:?}")
}

pub fn new(root: PathBuf) -> Result<ModuleBox> {
    let mut ctx = engine::Context::new();
    Ok(ConfigurationStatement::parse(root)?
        .eval(&mut ctx)?
        .unwrap_or_else(module::dummy_box))
}

pub fn verify(root: &Path) -> Result<()> {
    ConfigurationStatement::verify(root)
}

#[derive(Debug)]
struct ConfigurationStatement {
    root: PathBuf,
    statements: Vec<StatementBox>,
}

impl Statement for ConfigurationStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        let mut modules: Vec<_> = Vec::new();
        for statement in self.statements.iter() {
            if !ctx.enabled {
                break;
            }
            if let Some(module) = statement.eval(ctx)? {
                modules.push(module);
            }
        }
        if modules.is_empty() {
            return Ok(None);
        }
        Ok(Some(module::wrap(modules, error_context(&self.root))))
    }
}

// Analogous to engine::CommandBuilder, but can only be called from code.
impl ConfigurationStatement {
    pub fn parse(root: PathBuf) -> Result<StatementBox> {
        let manifest = root.join(MANIFEST);
        let statements = parser::parse(&root, &manifest)
            .with_context(|| format!("failed to load {manifest:?}"))?;
        Ok(Box::new(ConfigurationStatement { root, statements }))
    }
    pub fn verify(root: &Path) -> Result<()> {
        let manifest = root.join(MANIFEST);
        let _statements = parser::parse(root, &manifest)
            .with_context(|| format!("failed to load {manifest:?}"))?;
        Ok(())
    }
}
