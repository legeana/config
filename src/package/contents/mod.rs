mod builder;
mod cat_glob;
mod copy;
mod deprecated;
mod exec;
mod fetch;
mod file_util;
mod git_clone;
mod if_missing;
mod if_os;
mod importer;
mod local_state;
mod mkdir;
mod output_file;
mod parser;
mod prefix;
mod set_contents;
mod subdir;
mod symlink;
mod symlink_tree;
mod tags;
mod util;
mod xdg_prefix;

use core::fmt;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::package::{Module, Rules};
use crate::registry::Registry;

const MANIFEST: &str = "MANIFEST";

pub use builder::help;

pub struct Configuration {
    root: PathBuf,
    modules: Vec<Box<dyn Module>>,
}

impl Configuration {
    pub fn new_empty(root: PathBuf) -> Self {
        Self {
            root,
            modules: Vec::default(),
        }
    }
    pub fn new(root: PathBuf) -> Result<Self> {
        let mut state = builder::State::new(root.clone());
        Self::new_sub(&mut state, root)
    }
    pub fn new_sub(state: &mut builder::State, root: PathBuf) -> Result<Self> {
        let manifest = root.join(MANIFEST);
        let builders =
            parser::parse(&manifest).with_context(|| format!("failed to load {manifest:?}"))?;
        let mut modules: Vec<_> = Vec::new();
        for builder in builders.iter() {
            if !state.enabled {
                break;
            }
            if let Some(module) = builder.build(state)? {
                modules.push(module);
            }
        }
        Ok(Self { root, modules })
    }
}

impl Module for Configuration {
    fn pre_install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        self.modules
            .pre_install(rules, registry)
            .with_context(|| format!("failed pre_install in {:?}", self.root))
    }
    fn install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        self.modules
            .install(rules, registry)
            .with_context(|| format!("failed install in {:?}", self.root))
    }
    fn post_install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        self.modules
            .post_install(rules, registry)
            .with_context(|| format!("failed post_install in {:?}", self.root))
    }
}

impl fmt::Display for Configuration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.root.display())
    }
}
