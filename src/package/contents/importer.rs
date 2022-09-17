use std::io::{BufRead, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::{fs::File, io::Write};

use super::file_util;
use super::local_state;
use super::parser;
use super::util;
use crate::registry::Registry;

use anyhow::{anyhow, Context, Result};
use walkdir::WalkDir;

pub struct ImporterParser {}

const COMMAND: &str = "import_from";

struct ImporterInstaller {
    dst: PathBuf,
}

struct ImporterHook {
    prefix: PathBuf,
    src: PathBuf,
    dst: PathBuf,
    output: PathBuf,
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

impl super::FileInstaller for ImporterInstaller {
    fn install(&self, registry: &mut dyn Registry) -> Result<()> {
        file_util::make_local_state(registry, &self.dst)?;
        Ok(())
    }
}

impl super::Hook for ImporterHook {
    fn execute(&self) -> Result<()> {
        let f = File::create(&self.output)
            .with_context(|| format!("failed to open {:?}", self.output))?;
        let mut out = BufWriter::new(f);
        render(&self.prefix, &self.src, &mut out).with_context(|| {
            format!(
                "failed to render state {:?} for {:?}",
                self.output, self.dst,
            )
        })
    }
}

impl parser::Parser for ImporterParser {
    fn name(&self) -> &'static str {
        COMMAND
    }
    fn help(&self) -> &'static str {
        "import_from <filename>
           create a symlink for filename in prefix to a local persistent state"
    }
    fn parse(
        &self,
        state: &mut parser::State,
        configuration: &mut super::Configuration,
        args: &[&str],
    ) -> parser::Result<()> {
        let filename = util::single_arg(COMMAND, args)?;
        let src = configuration.root.join(filename);
        let dst = state.prefix.current.join(filename);
        let prefix = dst
            .parent()
            .ok_or_else(|| anyhow!("failed to get parent of {dst:?}"))?;
        let output = local_state::state_path(&dst)
            .with_context(|| format!("failed to get state_path for {dst:?}"))?;
        configuration
            .files
            .push(Box::new(ImporterInstaller { dst: dst.clone() }));
        configuration.post_hooks.push(Box::new(ImporterHook {
            prefix: prefix.to_owned(),
            src,
            dst,
            output,
        }));
        Ok(())
    }
}
