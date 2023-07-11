use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::module::ModuleBox;

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
                // pass name/args separately
                let mut args: Vec<&str> = Vec::with_capacity(cmd.args.len() + 1);
                args.push(&cmd.name);
                args.extend(cmd.args.iter().map(String::as_str));
                builders.push(parse_statement(
                    cmd.location.line_number,
                    workdir,
                    manifest_path,
                    &args,
                )?);
            }
        }
    }
    Ok(builders)
}

pub fn parse_statement(
    line_num: usize,
    workdir: &Path,
    manifest_path: &Path,
    args: &[&str],
) -> Result<engine::StatementBox> {
    let line = shlex::join(args.into_iter().cloned()); // TODO: use the actual source
    let statement = engine::parse(workdir, args).with_context(|| {
        format!("failed to parse line {line_num} {line:?} from {manifest_path:?}")
    })?;
    Ok(Box::new(ParsedStatement {
        line,
        line_num,
        manifest_path: manifest_path.to_owned(),
        statement,
    }))
}
