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

use anyhow::{anyhow, Context, Result};

use crate::package::contents::builder::Builder;
use crate::package::{Module, Rules};
use crate::registry::Registry;

const MANIFEST: &str = "MANIFEST";

pub use builder::help;

pub struct Configuration {
    root: PathBuf,
    modules: Vec<Box<dyn Module>>,
}

impl Configuration {
    pub fn new_empty(root: PathBuf) -> Box<dyn Module> {
        Box::new(Self {
            root,
            modules: Vec::default(),
        })
    }
    #[allow(clippy::new_ret_no_self)]
    pub fn new(root: PathBuf) -> Result<Box<dyn Module>> {
        let mut state = builder::State::new(root.clone());
        Self::new_sub(&mut state, root)
    }
    pub fn new_sub(state: &mut builder::State, root: PathBuf) -> Result<Box<dyn Module>> {
        ConfigurationBuilder::parse(root)?
            .build(state)?
            .ok_or_else(|| anyhow!("failed to unwrap Configuration"))
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

#[derive(Debug)]
struct ConfigurationBuilder {
    root: PathBuf,
    builders: Vec<Box<dyn builder::Builder>>,
}

impl Builder for ConfigurationBuilder {
    fn build(&self, state: &mut builder::State) -> Result<Option<Box<dyn Module>>> {
        let mut modules: Vec<_> = Vec::new();
        for builder in self.builders.iter() {
            if !state.enabled {
                break;
            }
            if let Some(module) = builder.build(state)? {
                modules.push(module);
            }
        }
        Ok(Some(Box::new(Configuration {
            root: self.root.clone(),
            modules,
        })))
    }
}

// Analogous to builder::Parser, but can only be called from code.
impl ConfigurationBuilder {
    pub fn parse(root: PathBuf) -> Result<Box<dyn Builder>> {
        let manifest = root.join(MANIFEST);
        let builders = parser::parse(&root, &manifest)
            .with_context(|| format!("failed to load {manifest:?}"))?;
        Ok(Box::new(ConfigurationBuilder { root, builders }))
    }
}
