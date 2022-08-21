use std::collections::hash_map::HashMap;
use std::path::PathBuf;

use anyhow::Result;

pub trait Hook {
    // TODO
}

pub trait FileInstaller {
    // TODO
}

pub struct Configuration {
    root: PathBuf,
    subdirs: HashMap<String, Configuration>,
    pre_hooks: Vec<Box<dyn Hook>>,
    post_hooks: Vec<Box<dyn Hook>>,
    files: Vec<Box<dyn FileInstaller>>,
}

impl Configuration {
    pub fn new(root: PathBuf) -> Result<Self> {
        Ok(Configuration {
            root,
            subdirs: HashMap::new(),
            pre_hooks: Vec::new(),
            post_hooks: Vec::new(),
            files: Vec::new(),
        })
    }
}
