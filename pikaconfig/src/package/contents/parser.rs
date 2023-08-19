use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

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
struct Condition {
    manifest_path: PathBuf,
    line: String,
    cond: engine::ConditionBox,
}

impl engine::Condition for Condition {
    fn eval(&self, ctx: &engine::Context) -> Result<bool> {
        self.cond.eval(ctx).with_context(|| {
            format!(
                "failed to evaluate {manifest_path:?}: {line:?}",
                manifest_path = self.manifest_path,
                line = self.line,
            )
        })
    }
}

#[derive(Debug)]
struct NotCondition(engine::ConditionBox);

impl engine::Condition for NotCondition {
    fn eval(&self, ctx: &engine::Context) -> Result<bool> {
        Ok(!self.0.eval(ctx)?)
    }
}

#[derive(Debug)]
struct IfClauseStatement {
    cond: engine::ConditionBox,
    statements: VecStatement,
}

impl IfClauseStatement {
    /// Returns Some(_) if condition is true, None otherwise (try next IfClause).
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<Option<ModuleBox>>> {
        if self.cond.eval(ctx)? {
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

#[derive(Debug)]
struct AssignmentStatement {
    manifest_path: PathBuf,
    line: String,
    var_name: String,
    expression: engine::ExpressionBox,
}

impl engine::Statement for AssignmentStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        let engine::ExpressionOutput { module, output } = self.expression.eval(ctx)?;
        ctx.set_var(self.var_name.clone(), output)
            .with_context(|| {
                format!(
                    "failed to set variable {var} {manifest_path:?}: {line:?}",
                    var = self.var_name,
                    manifest_path = self.manifest_path,
                    line = self.line
                )
            })?;
        Ok(module)
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
                ast::Statement::Assignment(assignment) => {
                    parse_assignment(workdir, manifest_path, assignment)
                }
                ast::Statement::WithStatement(_) => {
                    todo!()
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
    let command = engine::new_command(workdir, &cmd.name, &cmd.args)
        .with_context(|| format!("failed to parse {manifest_path:?} line: {line}"))?;
    let engine::Command::Statement(statement) = command else {
        bail!("{name} is an expression and returns a value, use it in `var = {name} ...` context", name=cmd.name);
    };
    Ok(Box::new(ParsedStatement {
        manifest_path: manifest_path.to_owned(),
        line,
        statement,
    }))
}

fn parse_assignment(
    workdir: &Path,
    manifest_path: &Path,
    assignment: &ast::Assignment,
) -> Result<engine::StatementBox> {
    let cmd = &assignment.command;
    let line = cmd.to_string();
    let command = engine::new_command(workdir, &cmd.name, &cmd.args)
        .with_context(|| format!("failed to parse {manifest_path:?} line: {line}"))?;
    let engine::Command::Expression(expression) = command else {
        bail!("{name} is a statement and doesn't return a value, remove `{var} = ...`", name=cmd.name, var=assignment.var);
    };
    Ok(Box::new(AssignmentStatement {
        manifest_path: manifest_path.to_owned(),
        line,
        var_name: assignment.var.clone(),
        expression,
    }))
}

fn parse_condition(
    workdir: &Path,
    manifest_path: &Path,
    condition: &ast::Condition,
) -> Result<engine::ConditionBox> {
    match condition {
        ast::Condition::Command(cmd) => {
            let line = cmd.to_string();
            let cond = engine::new_condition(workdir, &cmd.name, &cmd.args)
                .with_context(|| format!("failed to parse {manifest_path:?} line: {line}"))?;
            Ok(Box::new(Condition {
                manifest_path: manifest_path.to_owned(),
                line,
                cond,
            }))
        }
        ast::Condition::Not(cond) => Ok(Box::new(NotCondition(parse_condition(
            workdir,
            manifest_path,
            cond,
        )?))),
    }
}

fn parse_if_clause(
    workdir: &Path,
    manifest_path: &Path,
    if_clause: &ast::IfClause,
) -> Result<IfClauseStatement> {
    let cond = parse_condition(workdir, manifest_path, &if_clause.condition)?;
    let statements = VecStatement(parse_statements(
        workdir,
        manifest_path,
        if_clause.statements.iter(),
    )?);
    Ok(IfClauseStatement { cond, statements })
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
