mod cat_glob;
mod copy;
mod deprecated;
mod exec;
mod fetch;
mod file_util;
mod git_clone;
mod importer;
mod local_state;
mod mkdir;
mod output_file;
mod parser;
mod prefix;
mod set_contents;
mod subdir;
mod subdirs;
mod symlink;
mod symlink_tree;
mod tags;
mod util;
mod xdg_prefix;

use core::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use anyhow::{anyhow, Context, Ok, Result};

use crate::package::Module;
use crate::registry::Registry;

const MANIFEST: &str = "MANIFEST";

pub use parser::help;

#[derive(Default)]
pub struct Configuration {
    enabled: bool,
    root: PathBuf,
    modules: Vec<Box<dyn Module>>,
}

impl Configuration {
    pub fn new_empty(root: PathBuf) -> Self {
        Self {
            enabled: false,
            root,
            ..Self::default()
        }
    }
    pub fn new(root: PathBuf) -> Result<Self> {
        let mut state = parser::State::new();
        Self::new_sub(&mut state, root)
    }
    pub fn new_sub(state: &mut parser::State, root: PathBuf) -> Result<Self> {
        let manifest = root.join(MANIFEST);
        let mut conf = Self {
            enabled: true,
            root,
            ..Self::default()
        };
        conf.parse(state, &manifest)
            .with_context(|| format!("failed to load {manifest:?}"))?;
        Ok(conf)
    }
    fn parse_line(&mut self, state: &mut parser::State, line: &str) -> Result<()> {
        if line.is_empty() || line.starts_with('#') {
            return Ok(());
        }
        let args = shlex::split(line).ok_or_else(|| anyhow!("failed to split line {:?}", line))?;
        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
        parser::parse(state, self, &arg_refs)
    }
    fn parse(&mut self, state: &mut parser::State, manifest_path: &PathBuf) -> Result<()> {
        let manifest = File::open(manifest_path)
            .with_context(|| format!("failed to open {manifest_path:?}"))?;
        let reader = BufReader::new(manifest);
        for (line_idx, line_or) in reader.lines().enumerate() {
            let line_num = line_idx + 1;
            let line = line_or.with_context(|| {
                format!("failed to read line {line_num} from {manifest_path:?}")
            })?;
            self.parse_line(state, &line).with_context(|| {
                format!("failed to parse line {line_num} {line:?} from {manifest_path:?}")
            })?;
        }
        Ok(())
    }
}

impl Module for Configuration {
    fn pre_install(&self, registry: &mut dyn Registry) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        self.modules
            .pre_install(registry)
            .with_context(|| format!("failed pre_install in {:?}", self.root))
    }
    fn install(&self, registry: &mut dyn Registry) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        self.modules
            .install(registry)
            .with_context(|| format!("failed install in {:?}", self.root))
    }
    fn post_install(&self, registry: &mut dyn Registry) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        self.modules
            .post_install(registry)
            .with_context(|| format!("failed post_install in {:?}", self.root))
    }
}

impl fmt::Display for Configuration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.root.display())
    }
}
