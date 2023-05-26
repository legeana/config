use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::{anyhow, Context, Result};

use crate::package::Module;

use super::builder;

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
    for (line_idx, line_or) in reader.lines().enumerate() {
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
