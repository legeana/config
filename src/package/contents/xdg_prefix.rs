use std::path::PathBuf;

use crate::package::contents::parser;
use crate::package::contents::util::single_arg;
use crate::package::contents::Configuration;

use anyhow::{Context, Result};

trait XdgPrefixParser {
    fn name(&self) -> &'static str;
    fn help(&self) -> &'static str;
    fn xdg_prefix(&self, path: &str) -> Result<PathBuf>;
}

pub struct XdgCachePrefixParser {}
impl XdgPrefixParser for XdgCachePrefixParser {
    fn name(&self) -> &'static str {
        "xdg_cache_prefix"
    }
    fn help(&self) -> &'static str {
        "xdg_cache_prefix <directory>
           set current installation prefix to $XDG_CACHE_HOME/<directory>"
    }
    fn xdg_prefix(&self, path: &str) -> Result<PathBuf> {
        let base = xdg::BaseDirectories::new().with_context(|| "failed to parse XDG_CACHE_HOME")?;
        Ok(base.get_cache_home().join(path))
    }
}

pub struct XdgConfigPrefixParser {}
impl XdgPrefixParser for XdgConfigPrefixParser {
    fn name(&self) -> &'static str {
        "xdg_config_prefix"
    }
    fn help(&self) -> &'static str {
        "xdg_config_prefix <directory>
           set current installation prefix to $XDG_CONFIG_HOME/<directory>"
    }
    fn xdg_prefix(&self, path: &str) -> Result<PathBuf> {
        let base =
            xdg::BaseDirectories::new().with_context(|| "failed to parse XDG_CONFIG_HOME")?;
        Ok(base.get_config_home().join(path))
    }
}

pub struct XdgDataPrefixParser {}
impl XdgPrefixParser for XdgDataPrefixParser {
    fn name(&self) -> &'static str {
        "xdg_data_prefix"
    }
    fn help(&self) -> &'static str {
        "xdg_data_prefix <directory>
           set current installation prefix to $XDG_DATA_HOME/<directory>"
    }
    fn xdg_prefix(&self, path: &str) -> Result<PathBuf> {
        let base = xdg::BaseDirectories::new().with_context(|| "failed to parse XDG_DATA_HOME")?;
        Ok(base.get_data_home().join(path))
    }
}

pub struct XdgStatePrefixParser {}
impl XdgPrefixParser for XdgStatePrefixParser {
    fn name(&self) -> &'static str {
        "xdg_state_prefix"
    }
    fn help(&self) -> &'static str {
        "xdg_state_prefix <directory>
           set current installation prefix to $XDG_STATE_HOME/<directory>"
    }
    fn xdg_prefix(&self, path: &str) -> Result<PathBuf> {
        let base = xdg::BaseDirectories::new().with_context(|| "failed to parse XDG_STATE_HOME")?;
        Ok(base.get_state_home().join(path))
    }
}

impl<T> parser::Parser for T
where
    T: XdgPrefixParser,
{
    fn name(&self) -> &'static str {
        self.name()
    }
    fn help(&self) -> &'static str {
        self.help()
    }
    fn parse(
        &self,
        state: &mut parser::State,
        _configuration: &mut Configuration,
        args: &[&str],
    ) -> parser::Result<()> {
        let path = single_arg(self.name(), args)?;
        state.prefix.set(self.xdg_prefix(path)?);
        Ok(())
    }
}
