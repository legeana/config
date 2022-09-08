use std::io::{BufRead, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::{fs::File, io::Write};

use crate::package::configuration::file_util::make_local_state;
use crate::package::configuration::parser;
use crate::package::configuration::util::single_arg;
use crate::package::configuration::Configuration;
use crate::registry::Registry;

use anyhow::{anyhow, Context, Result};
use walkdir::WalkDir;

pub struct ImporterParser {}

const COMMAND: &str = "import_from";

struct ImporterInstaller {
    src: PathBuf,
    dst: PathBuf,
}

/// Returns true if parser matched.
fn parse_import<W: Write>(prefix: &Path, line: &str, out: &mut W) -> Result<bool> {
    const COMMAND: &str = "#import ";
    if !line.starts_with(COMMAND) {
        return Ok(false);
    }
    let arg = line[COMMAND.len()..].trim();
    let include_file = prefix.join(arg);
    let subprefix = include_file.parent().ok_or(anyhow!(
        "failed to get parent of {}",
        include_file.display()
    ))?;
    render_w(subprefix, &include_file, out).with_context(|| format!("failed to import {}", arg))?;
    return Ok(true);
}

/// Returns true if parser matched.
fn parse_import_tree<W: Write>(prefix: &Path, line: &str, out: &mut W) -> Result<bool> {
    const COMMAND: &str = "#import_tree ";
    if !line.starts_with(COMMAND) {
        return Ok(false);
    }
    let arg = line[COMMAND.len()..].trim();
    let subdir = prefix.join(arg);
    for e in WalkDir::new(&subdir).sort_by_file_name() {
        let entry = e.with_context(|| format!("failed to read {}", subdir.display()))?;
        let include_file = entry.path();
        let subprefix = include_file.parent().ok_or(anyhow!(
            "failed to get parent of {}",
            include_file.display()
        ))?;
        render_w(subprefix, &include_file, out)
            .with_context(|| format!("failed to import tree {}", arg))?;
    }
    return Ok(true);
}

fn parse_line<W: Write>(prefix: &Path, line: &str, out: &mut W) -> Result<()> {
    if parse_import(prefix, line, out)? {
        return Ok(());
    }
    if parse_import_tree(prefix, line, out)? {
        return Ok(());
    }
    writeln!(out, "{}", line).with_context(|| format!("failed to write line"))?;
    return Ok(());
}

fn render_w<W: Write>(prefix: &Path, src: &Path, out: &mut W) -> Result<()> {
    let f = File::open(src).with_context(|| format!("failed to open {}", src.display()))?;
    let inp = BufReader::new(f);
    for line in inp.lines() {
        let line = line.with_context(|| format!("failed to read line from {}", src.display()))?;
        parse_line(prefix, &line, out).with_context(|| format!("failed to parse {}", line))?;
    }
    return Ok(());
}

fn render(src: &Path, dst: &Path) -> Result<()> {
    let f = File::create(dst).with_context(|| format!("failed to open {}", dst.display()))?;
    let mut out = BufWriter::new(f);
    let prefix = dst
        .parent()
        .ok_or(anyhow!("failed to get parent of {}", dst.display()))?;
    return render_w(prefix, src, &mut out)
        .with_context(|| format!("failed to render {}", dst.display()));
}

impl super::FileInstaller for ImporterInstaller {
    fn install(&self, registry: &mut dyn Registry) -> Result<()> {
        make_local_state(&self.dst)?;
        registry.register_symlink(&self.dst)?;
        render(&self.src, &self.dst)?;
        return Ok(());
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
        configuration: &mut Configuration,
        args: &[&str],
    ) -> parser::Result<()> {
        let filename = single_arg(COMMAND, args)?;
        configuration.files.push(Box::new(ImporterInstaller {
            src: configuration.root.join(filename),
            dst: state.prefix.current.join(filename),
        }));
        return Ok(());
    }
}
