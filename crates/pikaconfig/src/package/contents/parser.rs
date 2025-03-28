use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};

use crate::module::BoxedModule;

use super::args::Argument;
use super::ast;
use super::engine;
use super::engine::Statement as _;

#[derive(Debug)]
struct ParsedStatement {
    manifest_path: PathBuf,
    line: String,
    statement: engine::BoxedStatement,
}

impl engine::Statement for ParsedStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<BoxedModule>> {
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
    cond: engine::BoxedCondition,
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
struct NotCondition(engine::BoxedCondition);

impl engine::Condition for NotCondition {
    fn eval(&self, ctx: &engine::Context) -> Result<bool> {
        Ok(!self.0.eval(ctx)?)
    }
}

#[derive(Debug)]
struct IfClauseStatement {
    cond: engine::BoxedCondition,
    statements: engine::VecStatement,
}

enum IfClauseAction {
    Execute(Option<BoxedModule>),
    TryNext,
}

impl IfClauseStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<IfClauseAction> {
        if self.cond.eval(ctx)? {
            let opt_mod = self.statements.eval(ctx)?;
            Ok(IfClauseAction::Execute(opt_mod))
        } else {
            Ok(IfClauseAction::TryNext)
        }
    }
}

#[derive(Debug)]
struct IfStatement {
    if_clauses: Vec<IfClauseStatement>,
    else_clause: engine::VecStatement,
}

impl engine::Statement for IfStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<BoxedModule>> {
        for if_clause in &self.if_clauses {
            match if_clause.eval(ctx)? {
                IfClauseAction::Execute(opt_mod) => return Ok(opt_mod),
                IfClauseAction::TryNext => continue,
            }
        }
        self.else_clause.eval(ctx)
    }
}

#[derive(Debug)]
struct CommandAssignmentStatement {
    manifest_path: PathBuf,
    line: String,
    var_name: String,
    expression: engine::BoxedExpression,
}

impl engine::Statement for CommandAssignmentStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<BoxedModule>> {
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

#[derive(Debug)]
struct ValueAssignmentStatement {
    manifest_path: PathBuf,
    var_name: String,
    value: Argument,
}

impl engine::Statement for ValueAssignmentStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<BoxedModule>> {
        let value = ctx.expand_arg(&self.value).with_context(|| {
            format!(
                "failed to expand assignment {var} value {value:?} in {manifest_path:?}",
                var = self.var_name,
                manifest_path = self.manifest_path,
                value = self.value,
            )
        })?;
        ctx.set_var(self.var_name.clone(), value).with_context(|| {
            format!(
                "failed to set variable {var} {manifest_path:?}: {value:?}",
                var = self.var_name,
                manifest_path = self.manifest_path,
                value = self.value,
            )
        })?;
        Ok(None)
    }
}

pub(super) fn parse(workdir: &Path, manifest_path: &Path) -> Result<Vec<engine::BoxedStatement>> {
    let manifest = std::fs::read_to_string(manifest_path)
        .with_context(|| format!("failed to read {manifest_path:?}"))?;
    let manifest_ast = ast::Manifest::parse(manifest_path, manifest)
        .with_context(|| format!("failed to parse {manifest_path:?}"))?;
    parse_statements(workdir, manifest_path, manifest_ast.statements.iter())
}

pub(super) fn parse_statements<'a>(
    workdir: &Path,
    manifest_path: &Path,
    statements: impl Iterator<Item = &'a ast::Statement>,
) -> Result<Vec<engine::BoxedStatement>> {
    statements
        .map(|statement| -> Result<engine::BoxedStatement> {
            match statement {
                ast::Statement::Command(cmd) => parse_command(workdir, manifest_path, cmd),
                ast::Statement::IfStatement(if_st) => {
                    parse_if_statement(workdir, manifest_path, if_st)
                }
                ast::Statement::CommandAssignment(assignment) => {
                    parse_command_assignment(workdir, manifest_path, assignment)
                }
                ast::Statement::ValueAssignment(assignment) => {
                    parse_value_assignment(workdir, manifest_path, assignment)
                }
                ast::Statement::WithStatement(with_st) => {
                    parse_with_statement(workdir, manifest_path, with_st)
                }
            }
        })
        .collect()
}

fn parse_command(
    workdir: &Path,
    manifest_path: &Path,
    cmd: &ast::Invocation,
) -> Result<engine::BoxedStatement> {
    let line = cmd.to_string();
    let command = engine::new_command(workdir, &cmd.name, &cmd.args)
        .with_context(|| format!("failed to parse {manifest_path:?} line: {line}"))?;
    let engine::Command::Statement(statement) = command else {
        bail!(
            "{name} is an expression and returns a value, use it in `var = {name} ...` context",
            name = cmd.name
        );
    };
    Ok(Box::new(ParsedStatement {
        manifest_path: manifest_path.to_owned(),
        line,
        statement,
    }))
}

fn parse_command_assignment(
    workdir: &Path,
    manifest_path: &Path,
    assignment: &ast::CommandAssignment,
) -> Result<engine::BoxedStatement> {
    let cmd = &assignment.command;
    let line = cmd.to_string();
    let command = engine::new_command(workdir, &cmd.name, &cmd.args)
        .with_context(|| format!("failed to parse {manifest_path:?} line: {line}"))?;
    let engine::Command::Expression(expression) = command else {
        bail!(
            "{name} is a statement and doesn't return a value, remove `{var} = ...`",
            name = cmd.name,
            var = assignment.var
        );
    };
    Ok(Box::new(CommandAssignmentStatement {
        manifest_path: manifest_path.to_owned(),
        line,
        var_name: assignment.var.clone(),
        expression,
    }))
}

fn parse_value_assignment(
    _workdir: &Path,
    manifest_path: &Path,
    assignment: &ast::ValueAssignment,
) -> Result<engine::BoxedStatement> {
    Ok(Box::new(ValueAssignmentStatement {
        manifest_path: manifest_path.to_owned(),
        var_name: assignment.var.clone(),
        value: assignment.value.clone(),
    }))
}

fn parse_condition(
    workdir: &Path,
    manifest_path: &Path,
    condition: &ast::Condition,
) -> Result<engine::BoxedCondition> {
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
    let statements = engine::VecStatement(parse_statements(
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
) -> Result<engine::BoxedStatement> {
    let mut if_clauses: Vec<IfClauseStatement> =
        Vec::with_capacity(if_st.else_if_clauses.len() + 1);
    if_clauses.push(parse_if_clause(workdir, manifest_path, &if_st.if_clause)?);
    for if_clause in &if_st.else_if_clauses {
        if_clauses.push(parse_if_clause(workdir, manifest_path, if_clause)?);
    }
    let else_clause = engine::VecStatement(parse_statements(
        workdir,
        manifest_path,
        if_st.else_statements.iter(),
    )?);
    Ok(Box::new(IfStatement {
        if_clauses,
        else_clause,
    }))
}

fn parse_with_statement(
    workdir: &Path,
    manifest_path: &Path,
    with_st: &ast::WithStatement,
) -> Result<engine::BoxedStatement> {
    let statement = Box::new(engine::VecStatement(parse_statements(
        workdir,
        manifest_path,
        with_st.statements.iter(),
    )?));
    let wrapper = engine::new_with_wrapper(
        workdir,
        &with_st.wrapper.name,
        &with_st.wrapper.args,
        statement,
    )?;
    Ok(wrapper)
}
