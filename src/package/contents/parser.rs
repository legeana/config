use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::iter::Enumerate;
use std::path::Path;

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
                },
                Some((line_idx, Err(err))) => {
                    self.end = true;
                    return Some((line_idx, Err(err.into())));
                },
                None => {
                    self.end = true;
                    break;
                },
            }
        }
        if buffer.is_empty() {
            return None;
        }
        Some((buf_line_num, Ok(buffer.join(""))))
    }
}

fn parse_line(state: &mut builder::State, line: &str) -> Result<Option<Box<dyn Module>>> {
    if line.is_empty() || line.starts_with('#') {
        return Ok(None);
    }
    let args = shlex::split(line).ok_or_else(|| anyhow!("failed to split line {:?}", line))?;
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    builder::build(state, &arg_refs)
}

pub fn parse(state: &mut builder::State, manifest_path: &Path) -> Result<Vec<Box<dyn Module>>> {
    let manifest =
        File::open(manifest_path).with_context(|| format!("failed to open {manifest_path:?}"))?;
    let reader = BufReader::new(manifest);
    let mut modules: Vec<Box<dyn Module>> = Vec::new();
    for (line_idx, line_or) in LineIterator::new(reader.lines()) {
        let line_num = line_idx + 1;
        let line = line_or
            .with_context(|| format!("failed to read line {line_num} from {manifest_path:?}"))?;
        if let Some(m) = parse_line(state, &line).with_context(|| {
            format!("failed to parse line {line_num} {line:?} from {manifest_path:?}")
        })? {
            modules.push(m);
        }
    }
    Ok(modules)
}