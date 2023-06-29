use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::iter::Enumerate;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use crate::package::Module;

use super::builder;

const LINE_CONT: char = '\\';

struct LineIterator<B: BufRead> {
    lines: Enumerate<Lines<B>>,
    end: bool,
}

impl<B: BufRead> LineIterator<B> {
    fn new(lines: Lines<B>) -> Self {
        Self {
            lines: lines.enumerate(),
            end: false,
        }
    }
}

fn is_cont(line: &str) -> bool {
    // TODO: support "line\\"
    line.ends_with(LINE_CONT)
}

impl<B: BufRead> Iterator for LineIterator<B> {
    type Item = (usize, Result<String>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.end {
            return None;
        }
        let mut buffer: Vec<String> = Vec::new();
        let mut buf_line_num = 0;
        loop {
            match self.lines.next() {
                Some((line_idx, Ok(line))) => {
                    if buffer.is_empty() {
                        buf_line_num = line_idx;
                    }
                    if is_cont(&line) {
                        buffer.push(line[..line.len() - 1].into());
                    } else {
                        buffer.push(line);
                        break;
                    }
                }
                Some((line_idx, Err(err))) => {
                    self.end = true;
                    return Some((line_idx, Err(err.into())));
                }
                None => {
                    self.end = true;
                    break;
                }
            }
        }
        if buffer.is_empty() {
            return None;
        }
        Some((buf_line_num, Ok(buffer.join(""))))
    }
}

fn parse_line(workdir: &Path, line: &str) -> Result<Option<Box<dyn builder::Statement>>> {
    if line.is_empty() || line.starts_with('#') {
        return Ok(None);
    }
    let args = shlex::split(line).ok_or_else(|| anyhow!("failed to split line {:?}", line))?;
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    Ok(Some(builder::parse(workdir, &arg_refs)?))
}

#[derive(Debug)]
struct ParsedStatement {
    line_num: usize,
    line: String,
    manifest_path: PathBuf,
    statement: Box<dyn builder::Statement>,
}

impl builder::Statement for ParsedStatement {
    fn eval(&self, state: &mut builder::State) -> Result<Option<Box<dyn Module>>> {
        self.statement.eval(state).with_context(|| {
            format!(
                "failed to build line {line_num} {line:?} from {manifest_path:?}",
                line_num = self.line_num,
                line = self.line,
                manifest_path = self.manifest_path,
            )
        })
    }
}

pub fn parse(workdir: &Path, manifest_path: &Path) -> Result<Vec<Box<dyn builder::Statement>>> {
    let manifest =
        File::open(manifest_path).with_context(|| format!("failed to open {manifest_path:?}"))?;
    let reader = BufReader::new(manifest);
    let mut builders: Vec<Box<dyn builder::Statement>> = Vec::new();
    for (line_idx, line_or) in LineIterator::new(reader.lines()) {
        let line_num = line_idx + 1;
        let line = line_or
            .with_context(|| format!("failed to read line {line_num} from {manifest_path:?}"))?;
        if let Some(statement) = parse_line(workdir, &line).with_context(|| {
            format!("failed to parse line {line_num} {line:?} from {manifest_path:?}")
        })? {
            builders.push(Box::new(ParsedStatement {
                line_num,
                line,
                manifest_path: manifest_path.to_owned(),
                statement,
            }));
        }
    }
    Ok(builders)
}
