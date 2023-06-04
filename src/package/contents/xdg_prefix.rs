use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::module::Module;

use super::builder;
use super::util;

trait XdgPrefixBuilder {
    fn name(&self) -> String;
    fn help(&self) -> String;
    fn xdg_prefix(&self, path: &str) -> Result<PathBuf>;
}

pub struct XdgCachePrefixBuilder;
impl XdgPrefixBuilder for XdgCachePrefixBuilder {
    fn name(&self) -> String {
        "xdg_cache_prefix".to_owned()
    }
    fn help(&self) -> String {
        "xdg_cache_prefix <directory>
           set current installation prefix to $XDG_CACHE_HOME/<directory>"
           .to_owned()
    }
    fn xdg_prefix(&self, path: &str) -> Result<PathBuf> {
        let base = xdg::BaseDirectories::new().context("failed to parse XDG_CACHE_HOME")?;
        Ok(base.get_cache_home().join(path))
    }
}

pub struct XdgConfigPrefixBuilder;
impl XdgPrefixBuilder for XdgConfigPrefixBuilder {
    fn name(&self) -> String {
        "xdg_config_prefix".to_owned()
    }
    fn help(&self) -> String {
        "xdg_config_prefix <directory>
           set current installation prefix to $XDG_CONFIG_HOME/<directory>"
           .to_owned()
    }
    fn xdg_prefix(&self, path: &str) -> Result<PathBuf> {
        let base = xdg::BaseDirectories::new().context("failed to parse XDG_CONFIG_HOME")?;
        Ok(base.get_config_home().join(path))
    }
}

pub struct XdgDataPrefixBuilder;
impl XdgPrefixBuilder for XdgDataPrefixBuilder {
    fn name(&self) -> String {
        "xdg_data_prefix".to_owned()
    }
    fn help(&self) -> String {
        "xdg_data_prefix <directory>
           set current installation prefix to $XDG_DATA_HOME/<directory>"
           .to_owned()
    }
    fn xdg_prefix(&self, path: &str) -> Result<PathBuf> {
        let base = xdg::BaseDirectories::new().context("failed to parse XDG_DATA_HOME")?;
        Ok(base.get_data_home().join(path))
    }
}

pub struct XdgStatePrefixBuilder;
impl XdgPrefixBuilder for XdgStatePrefixBuilder {
    fn name(&self) -> String {
        "xdg_state_prefix".to_owned()
    }
    fn help(&self) -> String {
        "xdg_state_prefix <directory>
           set current installation prefix to $XDG_STATE_HOME/<directory>"
           .to_owned()
    }
    fn xdg_prefix(&self, path: &str) -> Result<PathBuf> {
        let base = xdg::BaseDirectories::new().context("failed to parse XDG_STATE_HOME")?;
        Ok(base.get_state_home().join(path))
    }
}

impl<T> builder::Builder for T
where
    T: XdgPrefixBuilder,
{
    fn name(&self) -> String {
        self.name()
    }
    fn help(&self) -> String {
        self.help()
    }
    fn build(&self, state: &mut builder::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        let path = util::single_arg(&self.name(), args)?;
        state.prefix.set(self.xdg_prefix(path)?);
        Ok(None)
    }
}
