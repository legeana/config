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
mod which;

use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use lalrpop_util::lalrpop_mod;

use crate::module::{self, BoxedModule};
use crate::package::contents::engine::{BoxedStatement, Statement};

lalrpop_mod!(
    #[allow(clippy::pedantic)]
    #[allow(clippy::shadow_unrelated)]
    #[allow(clippy::use_self)]
    #[allow(unused_qualifications)]
    ast_parser,
    "/package/contents/ast_parser.rs"
);

const MANIFEST: &str = "MANIFEST";

pub use engine::help;

fn error_context(root: &Path) -> String {
    format!("{root:?}")
}

pub(super) fn new(root: PathBuf) -> Result<BoxedModule> {
    let mut ctx = engine::Context::new();
    Ok(ConfigurationStatement::parse(root)?
        .eval(&mut ctx)?
        .unwrap_or_else(module::dummy_box))
}

pub(super) fn verify(root: &Path) -> Result<()> {
    ConfigurationStatement::verify(root)
}

#[derive(Debug)]
struct ConfigurationStatement {
    root: PathBuf,
    statements: engine::VecStatement,
}

impl Statement for ConfigurationStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<BoxedModule>> {
        match self.statements.eval(ctx)? {
            Some(m) => Ok(Some(module::wrap(m, error_context(&self.root)))),
            None => Ok(None),
        }
    }
}

// Analogous to engine::CommandBuilder, but can only be called from code.
impl ConfigurationStatement {
    pub(super) fn parse(root: PathBuf) -> Result<BoxedStatement> {
        let manifest = root.join(MANIFEST);
        let statements = engine::VecStatement(
            parser::parse(&root, &manifest)
                .with_context(|| format!("failed to load {manifest:?}"))?,
        );
        Ok(Box::new(Self { root, statements }))
    }
    pub(super) fn verify(root: &Path) -> Result<()> {
        let manifest = root.join(MANIFEST);
        let _statements = parser::parse(root, &manifest)
            .with_context(|| format!("failed to load {manifest:?}"))?;
        Ok(())
    }
}
