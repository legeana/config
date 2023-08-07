use std::io::{BufRead, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::{fs::File, io::Write};

use crate::module::{Module, ModuleBox, Rules};
use crate::registry::Registry;

use super::args::{Argument, Arguments};
use super::engine;
use super::inventory;
use super::local_state;

use anyhow::{anyhow, Context, Result};
use indoc::formatdoc;
use walkdir::WalkDir;

struct Importer {
    prefix: PathBuf,
    src: PathBuf,
    output: local_state::FileState,
}

/// Returns true if parser matched.
fn parse_import<W: Write>(prefix: &Path, line: &str, out: &mut W) -> Result<bool> {
    const COMMAND: &str = "#import ";
    if !line.starts_with(COMMAND) {
        return Ok(false);
    }
    let arg = line[COMMAND.len()..].trim();
    let include_file = prefix.join(arg);
    let subprefix = include_file
        .parent()
        .ok_or_else(|| anyhow!("failed to get parent of {include_file:?}"))?;
    render(subprefix, &include_file, out).with_context(|| format!("failed to import {arg}"))?;
    Ok(true)
}

/// Returns true if parser matched.
fn parse_import_tree<W: Write>(prefix: &Path, line: &str, out: &mut W) -> Result<bool> {
    const COMMAND: &str = "#import_tree ";
    if !line.starts_with(COMMAND) {
        return Ok(false);
    }
    let arg = line[COMMAND.len()..].trim();
    let subdir = prefix.join(arg);
    log::trace!("#import_tree {subdir:?}");
    for e in WalkDir::new(&subdir).sort_by_file_name() {
        let entry = e.with_context(|| format!("failed to read {subdir:?}"))?;
        let md = std::fs::metadata(entry.path())
            .with_context(|| format!("failed to get {:?} metadata", entry.path()))?;
        if !md.file_type().is_file() {
            // Only files (or symlinks to files) are supported.
            continue;
        }
        let include_file = entry.path();
        let subprefix = include_file
            .parent()
            .ok_or_else(|| anyhow!("failed to get parent of {include_file:?}"))?;
        render(subprefix, include_file, out)
            .with_context(|| format!("failed to import tree {}", arg))?;
    }
    Ok(true)
}

fn parse_line<W: Write>(prefix: &Path, line: &str, out: &mut W) -> Result<()> {
    if parse_import(prefix, line, out)? {
        return Ok(());
    }
    if parse_import_tree(prefix, line, out)? {
        return Ok(());
    }
    writeln!(out, "{}", line).with_context(|| "failed to write line")?;
    Ok(())
}

fn render<W: Write>(prefix: &Path, src: &Path, out: &mut W) -> Result<()> {
    let f = File::open(src).with_context(|| format!("failed to open {src:?}"))?;
    let inp = BufReader::new(f);
    for line in inp.lines() {
        let line = line.with_context(|| format!("failed to read line from {src:?}"))?;
        parse_line(prefix, &line, out).with_context(|| format!("failed to parse {line}"))?;
    }
    Ok(())
}

impl Module for Importer {
    fn install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        self.output.install(rules, registry)
    }
    fn post_install(&self, _rules: &super::Rules, _registry: &mut dyn Registry) -> Result<()> {
        let f = File::create(self.output.path())
            .with_context(|| format!("failed to open {:?}", self.output.path()))?;
        let mut out = BufWriter::new(f);
        render(&self.prefix, &self.src, &mut out).with_context(|| {
            format!(
                "failed to render state {:?} for {:?}",
                self.output.path(),
                self.output.link(),
            )
        })
    }
}

#[derive(Debug)]
struct ImporterStatement {
    workdir: PathBuf,
    filename: Argument,
}

impl engine::Statement for ImporterStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        let dst = ctx.dst_path(ctx.expand_arg(&self.filename)?);
        let prefix = dst
            .parent()
            .ok_or_else(|| anyhow!("failed to get parent of {dst:?}"))?;
        let output = local_state::FileState::new(dst.clone())
            .with_context(|| format!("failed to create FileState for {dst:?}"))?;
        Ok(Some(Box::new(Importer {
            prefix: prefix.to_owned(),
            src: self.workdir.join(ctx.expand_arg(&self.filename)?),
            output,
        })))
    }
}

#[derive(Clone)]
struct ImporteBuilder;

impl engine::CommandBuilder for ImporteBuilder {
    fn name(&self) -> String {
        "import_from".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <filename>
                create a symlink for filename in prefix to a local persistent state
        ", command=self.name()}
    }
    fn build(&self, workdir: &Path, args: &Arguments) -> Result<engine::Command> {
        let filename = args.expect_single_arg(self.name())?.clone();
        Ok(engine::Command::new_statement(ImporterStatement {
            workdir: workdir.to_owned(),
            filename,
        }))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(ImporteBuilder {}));
}
