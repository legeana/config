use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::module::ModuleBox;

use super::args::Arguments;
use super::ast;
use super::engine;

#[derive(Debug)]
struct ParsedStatement {
    line_num: usize,
    line: String,
    manifest_path: PathBuf,
    statement: engine::StatementBox,
}

impl engine::Statement for ParsedStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        self.statement.eval(ctx).with_context(|| {
            format!(
                "failed to evaluate {manifest_path:?} line {line_num}: {line:?}",
                manifest_path = self.manifest_path,
                line_num = self.line_num,
                line = self.line,
            )
        })
    }
}

pub fn parse(workdir: &Path, manifest_path: &Path) -> Result<Vec<engine::StatementBox>> {
    let manifest = std::fs::read_to_string(manifest_path)
        .with_context(|| format!("failed to read {manifest_path:?}"))?;
    let mut builders: Vec<engine::StatementBox> = Vec::new();
    let manifest_ast = ast::Manifest::parse(manifest_path, manifest)
        .with_context(|| format!("failed to parse {manifest_path:?}"))?;
    for statement in manifest_ast.statements {
        match statement {
            ast::Statement::Command(cmd) => {
                builders.push(parse_statement(
                    cmd.location.line_number,
                    workdir,
                    manifest_path,
                    cmd.name,
                    &cmd.args,
                )?);
            }
            ast::Statement::IfStatement(_if_st) => {
                todo!();
            }
        }
    }
    Ok(builders)
}

pub fn parse_statement(
    line_num: usize,
    workdir: &Path,
    manifest_path: &Path,
    cmd: impl AsRef<str>,
    args: &Arguments,
) -> Result<engine::StatementBox> {
    // TODO: use the actual source
    let line = shlex::join(std::iter::once(cmd.as_ref()).chain(args.0.iter().map(String::as_str)));
    let statement = engine::parse(workdir, cmd.as_ref(), args).with_context(|| {
        format!("failed to parse line {line_num} {line:?} from {manifest_path:?}")
    })?;
    Ok(Box::new(ParsedStatement {
        line,
        line_num,
        manifest_path: manifest_path.to_owned(),
        statement,
    }))
}
