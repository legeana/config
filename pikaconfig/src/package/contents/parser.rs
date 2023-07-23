use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::module::ModuleBox;

use super::ast;
use super::engine;

#[derive(Debug)]
struct ParsedStatement {
    manifest_path: PathBuf,
    line: String,
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

#[derive(Debug)]
struct IfStatement {
    manifest_path: PathBuf,
    line: String,
    cond: engine::ConditionBox,
    if_true: VecStatement,
    if_false: VecStatement,
}

impl engine::Statement for IfStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        if self.cond.eval(ctx).with_context(|| {
            format!(
                "failed to evaluate {manifest_path:?}: {line:?}",
                manifest_path = self.manifest_path,
                line = self.line,
            )
        })? {
            self.if_true.eval(ctx)
        } else {
            self.if_false.eval(ctx)
        }
    }
}

#[derive(Debug)]
struct VecStatement(Vec<engine::StatementBox>);

impl engine::Statement for VecStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        let mut modules: Vec<ModuleBox> = Vec::new();
        for st in &self.0 {
            if let Some(m) = st.eval(ctx)? {
                modules.push(m);
            }
        }
        if modules.is_empty() {
            Ok(None)
        } else {
            Ok(Some(Box::new(modules)))
        }
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
                ast::Statement::Command(cmd) => parse_command(workdir, manifest_path, cmd),
                ast::Statement::IfStatement(if_st) => {
                    parse_if_statement(workdir, manifest_path, if_st)
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
        .with_context(|| format!("failed to parse {manifest_path:?} line: {line}"))?;
    Ok(Box::new(ParsedStatement {
        manifest_path: manifest_path.to_owned(),
        line,
        statement,
    }))
}

pub fn parse_if_statement(
    workdir: &Path,
    manifest_path: &Path,
    if_st: &ast::IfStatement,
) -> Result<engine::StatementBox> {
    let line = if_st.conditional.to_string();
    let cond = engine::new_condition(workdir, &if_st.conditional.name, &if_st.conditional.args)
        .with_context(|| format!("failed to parse {manifest_path:?} line: {line}"))?;
    let if_true = VecStatement(parse_statements(
        workdir,
        manifest_path,
        if_st.statements.iter(),
    )?);
    let if_false = VecStatement(parse_statements(
        workdir,
        manifest_path,
        if_st.else_statements.iter(),
    )?);
    Ok(Box::new(IfStatement {
        manifest_path: manifest_path.to_owned(),
        line,
        cond,
        if_true,
        if_false,
    }))
}
