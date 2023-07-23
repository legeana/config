use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::module::ModuleBox;

use super::ast;
use super::engine;

#[derive(Debug)]
struct ParsedStatement {
    line: String,
    manifest_path: PathBuf,
    statement: engine::StatementBox,
}

impl engine::Statement for ParsedStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        self.statement.eval(ctx).with_context(|| {
            format!(
                "failed to evaluate {manifest_path:?}: {line:?}",
                manifest_path = self.manifest_path,
                line = self.line,
            )
        })
    }
}

pub fn parse(workdir: &Path, manifest_path: &Path) -> Result<Vec<engine::StatementBox>> {
    let manifest = std::fs::read_to_string(manifest_path)
        .with_context(|| format!("failed to read {manifest_path:?}"))?;
    let manifest_ast = ast::Manifest::parse(manifest_path, manifest)
        .with_context(|| format!("failed to parse {manifest_path:?}"))?;
    parse_statements(workdir, manifest_path, manifest_ast.statements.iter())
}

pub fn parse_statements<'a>(
    workdir: &Path,
    manifest_path: &Path,
    statements: impl Iterator<Item = &'a ast::Statement>,
) -> Result<Vec<engine::StatementBox>> {
    statements
        .map(|statement| -> Result<engine::StatementBox> {
            match statement {
                ast::Statement::Command(cmd) => parse_command(workdir, manifest_path, &cmd),
                ast::Statement::IfStatement(_if_st) => {
                    todo!();
                }
            }
        })
        .collect()
}

pub fn parse_command(
    workdir: &Path,
    manifest_path: &Path,
    cmd: &ast::Invocation,
) -> Result<engine::StatementBox> {
    let line = cmd.to_string();
    let statement = engine::new_command(workdir, &cmd.name, &cmd.args)
        .with_context(|| format!("failed to parse line {manifest_path:?}: {line}"))?;
    Ok(Box::new(ParsedStatement {
        line,
        manifest_path: manifest_path.to_owned(),
        statement,
    }))
}
