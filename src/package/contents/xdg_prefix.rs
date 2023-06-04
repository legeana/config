use std::path::PathBuf;

use anyhow::{Context, Result};
use indoc::formatdoc;

use crate::module::Module;

use super::builder;
use super::util;

trait XdgPrefixBuilder {
    fn name(&self) -> &str;
    fn var(&self) -> &str;
    fn xdg_prefix(&self, path: &str) -> Result<PathBuf>;
}

pub struct XdgCachePrefixBuilder;
impl XdgPrefixBuilder for XdgCachePrefixBuilder {
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

pub struct XdgConfigPrefixBuilder;
impl XdgPrefixBuilder for XdgConfigPrefixBuilder {
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

pub struct XdgDataPrefixBuilder;
impl XdgPrefixBuilder for XdgDataPrefixBuilder {
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

pub struct XdgStatePrefixBuilder;
impl XdgPrefixBuilder for XdgStatePrefixBuilder {
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

impl<T> builder::Builder for T
where
    T: XdgPrefixBuilder,
{
    fn name(&self) -> String {
        self.name().to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <directory>
                set current installation prefix to ${var}/<directory>
        ", command=self.name(), var=self.var()}
    }
    fn build(&self, state: &mut builder::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        let path = util::single_arg(self.name(), args)?;
        state.prefix.set(self.xdg_prefix(path)?);
        Ok(None)
    }
}
