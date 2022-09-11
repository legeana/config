mod cat_glob;
mod copy;
mod deprecated;
mod exec;
mod file_util;
mod importer;
mod local_state;
mod mkdir;
mod output_file;
mod parser;
mod prefix;
mod subdir;
mod symlink;
mod symlink_tree;
mod tags;
mod util;
mod xdg_prefix;

use crate::registry::Registry;

use core::fmt;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::{collections::hash_map::HashMap, io::BufRead};

use anyhow::{anyhow, Context, Ok, Result};
use shlex;

const MANIFEST: &str = "MANIFEST";

pub use parser::help;

pub trait FileInstaller {
    fn install(&self, registry: &mut dyn Registry) -> Result<()>;
}

pub trait Hook {
    fn execute(&self) -> Result<()>;
}

pub struct Configuration {
    enabled: bool,
    root: PathBuf,
    subdirs: HashMap<String, Configuration>,
    pre_hooks: Vec<Box<dyn Hook>>,
    post_hooks: Vec<Box<dyn Hook>>,
    files: Vec<Box<dyn FileInstaller>>,
}

impl Configuration {
    pub fn new(root: PathBuf) -> Result<Self> {
        let mut state = parser::State::new();
        Configuration::new_sub(&mut state, root)
    }
    pub fn new_sub(state: &mut parser::State, root: PathBuf) -> Result<Self> {
        let manifest = root.join(MANIFEST);
        let mut conf = Configuration {
            enabled: true,
            root,
            subdirs: HashMap::new(),
            pre_hooks: Vec::new(),
            post_hooks: Vec::new(),
            files: Vec::new(),
        };
        conf.parse(state, &manifest)
            .with_context(|| format!("failed to load {}", manifest.display()))?;
        return Ok(conf);
    }
    fn parse_line(&mut self, state: &mut parser::State, line: &str) -> Result<()> {
        if line.is_empty() || line.starts_with("#") {
            return Ok(());
        }
        let args = shlex::split(&line).ok_or(anyhow!("failed to split line {:?}", line))?;
        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
        return parser::parse(state, self, &arg_refs);
    }
    fn parse(&mut self, state: &mut parser::State, manifest_path: &PathBuf) -> Result<()> {
        let manifest = File::open(manifest_path)
            .with_context(|| format!("failed to open {}", manifest_path.display()))?;
        let reader = BufReader::new(manifest);
        for (line_idx, line_or) in reader.lines().enumerate() {
            let line_num = line_idx + 1;
            let line = line_or.with_context(|| {
                format!(
                    "failed to read line {} from {}",
                    line_num,
                    manifest_path.display()
                )
            })?;
            self.parse_line(state, &line).with_context(|| {
                format!(
                    "failed to parse line {} {:?} from {}",
                    line_num,
                    line,
                    manifest_path.display()
                )
            })?;
        }
        return Ok(());
    }
    pub fn pre_install(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        for hook in self.pre_hooks.iter() {
            hook.execute()
                .with_context(|| format!("failed to execute pre-install hook"))?;
        }
        for (name, subdir) in self.subdirs.iter() {
            subdir
                .pre_install()
                .with_context(|| format!("failed to pre-install {name}"))?;
        }
        return Ok(());
    }
    pub fn install(&self, registry: &mut dyn Registry) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        for file in self.files.iter() {
            file.install(registry)
                .with_context(|| format!("failed to install file installer"))?;
        }
        for (name, subdir) in self.subdirs.iter() {
            subdir
                .install(registry)
                .with_context(|| format!("failed to install {name}"))?;
        }
        return Ok(());
    }
    pub fn post_install(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        for hook in self.post_hooks.iter() {
            hook.execute()
                .with_context(|| format!("failed to execute post-install hook"))?;
        }
        for (name, subdir) in self.subdirs.iter() {
            subdir
                .post_install()
                .with_context(|| format!("failed to post-install {name}"))?;
        }
        return Ok(());
    }
}

impl fmt::Display for Configuration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.root.display())
    }
}
