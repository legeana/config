use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::module::Module;

use super::parser;
use super::util;

trait XdgPrefixBuilder {
    fn name(&self) -> &'static str;
    fn help(&self) -> &'static str;
    fn xdg_prefix(&self, path: &str) -> Result<PathBuf>;
}

pub struct XdgCachePrefixBuilder;
impl XdgPrefixBuilder for XdgCachePrefixBuilder {
    fn name(&self) -> &'static str {
        "xdg_cache_prefix"
    }
    fn help(&self) -> &'static str {
        "xdg_cache_prefix <directory>
           set current installation prefix to $XDG_CACHE_HOME/<directory>"
    }
    fn xdg_prefix(&self, path: &str) -> Result<PathBuf> {
        let base = xdg::BaseDirectories::new().context("failed to parse XDG_CACHE_HOME")?;
        Ok(base.get_cache_home().join(path))
    }
}

pub struct XdgConfigPrefixBuilder;
impl XdgPrefixBuilder for XdgConfigPrefixBuilder {
    fn name(&self) -> &'static str {
        "xdg_config_prefix"
    }
    fn help(&self) -> &'static str {
        "xdg_config_prefix <directory>
           set current installation prefix to $XDG_CONFIG_HOME/<directory>"
    }
    fn xdg_prefix(&self, path: &str) -> Result<PathBuf> {
        let base = xdg::BaseDirectories::new().context("failed to parse XDG_CONFIG_HOME")?;
        Ok(base.get_config_home().join(path))
    }
}

pub struct XdgDataPrefixBuilder;
impl XdgPrefixBuilder for XdgDataPrefixBuilder {
    fn name(&self) -> &'static str {
        "xdg_data_prefix"
    }
    fn help(&self) -> &'static str {
        "xdg_data_prefix <directory>
           set current installation prefix to $XDG_DATA_HOME/<directory>"
    }
    fn xdg_prefix(&self, path: &str) -> Result<PathBuf> {
        let base = xdg::BaseDirectories::new().context("failed to parse XDG_DATA_HOME")?;
        Ok(base.get_data_home().join(path))
    }
}

pub struct XdgStatePrefixBuilder;
impl XdgPrefixBuilder for XdgStatePrefixBuilder {
    fn name(&self) -> &'static str {
        "xdg_state_prefix"
    }
    fn help(&self) -> &'static str {
        "xdg_state_prefix <directory>
           set current installation prefix to $XDG_STATE_HOME/<directory>"
    }
    fn xdg_prefix(&self, path: &str) -> Result<PathBuf> {
        let base = xdg::BaseDirectories::new().context("failed to parse XDG_STATE_HOME")?;
        Ok(base.get_state_home().join(path))
    }
}

impl<T> parser::Builder for T
where
    T: XdgPrefixBuilder,
{
    fn name(&self) -> &'static str {
        self.name()
    }
    fn help(&self) -> &'static str {
        self.help()
    }
    fn parse(&self, state: &mut parser::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        let path = util::single_arg(self.name(), args)?;
        state.prefix.set(self.xdg_prefix(path)?);
        Ok(None)
    }
}
