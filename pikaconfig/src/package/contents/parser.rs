use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::module::ModuleBox;

use super::ast;
use super::engine;
use super::engine::Statement;

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
struct IfClauseStatement {
    manifest_path: PathBuf,
    line: String,
    cond: engine::ConditionBox,
    statements: VecStatement,
}

impl IfClauseStatement {
    /// Returns Some(_) if condition is true, None otherwise (try next IfClause).
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<Option<ModuleBox>>> {
        if self.cond.eval(ctx).with_context(|| {
            format!(
                "failed to evaluate {manifest_path:?}: {line:?}",
                manifest_path = self.manifest_path,
                line = self.line,
            )
        })? {
            let opt_mod = self.statements.eval(ctx)?;
            Ok(Some(opt_mod))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug)]
struct IfStatement {
    if_clauses: Vec<IfClauseStatement>,
    else_clause: VecStatement,
}

impl engine::Statement for IfStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        for if_clause in &self.if_clauses {
            if let Some(opt_mod) = if_clause.eval(ctx)? {
                return Ok(opt_mod);
            }
        }
        self.else_clause.eval(ctx)
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

fn parse_command(
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

fn parse_if_clause(
    workdir: &Path,
    manifest_path: &Path,
    if_clause: &ast::IfClause,
) -> Result<IfClauseStatement> {
    let line = if_clause.condition.to_string();
    let cond = engine::new_condition(
        workdir,
        &if_clause.condition.name,
        &if_clause.condition.args,
    )
    .with_context(|| format!("failed to parse {manifest_path:?} line: {line}"))?;
    let statements = VecStatement(parse_statements(
        workdir,
        manifest_path,
        if_clause.statements.iter(),
    )?);
    Ok(IfClauseStatement {
        manifest_path: manifest_path.to_owned(),
        line,
        cond,
        statements,
    })
}

fn parse_if_statement(
    workdir: &Path,
    manifest_path: &Path,
    if_st: &ast::IfStatement,
) -> Result<engine::StatementBox> {
    let mut if_clauses: Vec<IfClauseStatement> =
        Vec::with_capacity(if_st.else_if_clauses.len() + 1);
    if_clauses.push(parse_if_clause(workdir, manifest_path, &if_st.if_clause)?);
    for if_clause in &if_st.else_if_clauses {
        if_clauses.push(parse_if_clause(workdir, manifest_path, if_clause)?);
    }
    let else_clause = VecStatement(parse_statements(
        workdir,
        manifest_path,
        if_st.else_statements.iter(),
    )?);
    Ok(Box::new(IfStatement {
        if_clauses,
        else_clause,
    }))
}
