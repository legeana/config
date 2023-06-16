use std::path::PathBuf;

use anyhow::{Context, Result};
use indoc::formatdoc;

use crate::module::Module;

use super::builder;
use super::util;

trait XdgPrefix {
    fn name(&self) -> &str;
    fn var(&self) -> &str;
    fn xdg_prefix(&self, path: &str) -> Result<PathBuf>;
}

#[derive(Clone)]
struct XdgCachePrefix;
impl XdgPrefix for XdgCachePrefix {
    fn name(&self) -> &str {
        "xdg_cache_prefix"
    }
    fn var(&self) -> &str {
        "XDG_CACHE_HOME"
    }
    fn xdg_prefix(&self, path: &str) -> Result<PathBuf> {
        let base = xdg::BaseDirectories::new().context("failed to parse XDG_CACHE_HOME")?;
        Ok(base.get_cache_home().join(path))
    }
}

#[derive(Clone)]
struct XdgConfigPrefix;
impl XdgPrefix for XdgConfigPrefix {
    fn name(&self) -> &str {
        "xdg_config_prefix"
    }
    fn var(&self) -> &str {
        "XDG_CONFIG_HOME"
    }
    fn xdg_prefix(&self, path: &str) -> Result<PathBuf> {
        let base = xdg::BaseDirectories::new().context("failed to parse XDG_CONFIG_HOME")?;
        Ok(base.get_config_home().join(path))
    }
}

#[derive(Clone)]
struct XdgDataPrefix;
impl XdgPrefix for XdgDataPrefix {
    fn name(&self) -> &str {
        "xdg_data_prefix"
    }
    fn var(&self) -> &str {
        "XDG_DATA_HOME"
    }
    fn xdg_prefix(&self, path: &str) -> Result<PathBuf> {
        let base = xdg::BaseDirectories::new().context("failed to parse XDG_DATA_HOME")?;
        Ok(base.get_data_home().join(path))
    }
}

#[derive(Clone)]
struct XdgStatePrefix;
impl XdgPrefix for XdgStatePrefix {
    fn name(&self) -> &str {
        "xdg_state_prefix"
    }
    fn var(&self) -> &str {
        "XDG_STATE_HOME"
    }
    fn xdg_prefix(&self, path: &str) -> Result<PathBuf> {
        let base = xdg::BaseDirectories::new().context("failed to parse XDG_STATE_HOME")?;
        Ok(base.get_state_home().join(path))
    }
}

#[derive(Clone)]
struct XdgPrefixBuilder<T>(T)
where
    T: XdgPrefix + Clone + 'static;

impl<T> builder::Builder for XdgPrefixBuilder<T>
where
    T: XdgPrefix + Clone + 'static,
{
    fn name(&self) -> String {
        self.0.name().to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <directory>
                set current installation prefix to ${var}/<directory>
        ", command=self.name(), var=self.0.var()}
    }
    fn build(&self, state: &mut builder::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        let path = util::single_arg(self.0.name(), args)?;
        state.prefix.set(self.0.xdg_prefix(path)?);
        Ok(None)
    }
}

pub fn commands() -> Vec<Box<dyn builder::Parser>> {
    vec![
        Box::new(XdgPrefixBuilder(XdgCachePrefix {})),
        Box::new(XdgPrefixBuilder(XdgConfigPrefix {})),
        Box::new(XdgPrefixBuilder(XdgDataPrefix {})),
        Box::new(XdgPrefixBuilder(XdgStatePrefix {})),
    ]
}
