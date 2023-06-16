use std::path::PathBuf;

use anyhow::Result;
use indoc::formatdoc;

#[cfg(unix)]
use anyhow::Context;

#[cfg(windows)]
use anyhow::anyhow;

use crate::module::Module;

use super::builder;
use super::util;

trait XdgPrefix: std::fmt::Debug {
    fn name(&self) -> &str;
    fn var(&self) -> &str;
    fn xdg_prefix(&self, path: &str) -> Result<PathBuf>;
}

#[derive(Clone, Debug)]
struct XdgCachePrefix;
impl XdgPrefix for XdgCachePrefix {
    fn name(&self) -> &str {
        "xdg_cache_prefix"
    }
    fn var(&self) -> &str {
        "XDG_CACHE_HOME"
    }
    #[cfg(unix)]
    fn xdg_prefix(&self, path: &str) -> Result<PathBuf> {
        let base = xdg::BaseDirectories::new().context("failed to parse XDG_CACHE_HOME")?;
        Ok(base.get_cache_home().join(path))
    }
    #[cfg(windows)]
    fn xdg_prefix(&self, _path: &str) -> Result<PathBuf> {
        Err(anyhow!("{} is not supported on Windows", self.name()))
    }
}

#[derive(Clone, Debug)]
struct XdgConfigPrefix;
impl XdgPrefix for XdgConfigPrefix {
    fn name(&self) -> &str {
        "xdg_config_prefix"
    }
    fn var(&self) -> &str {
        "XDG_CONFIG_HOME"
    }
    #[cfg(unix)]
    fn xdg_prefix(&self, path: &str) -> Result<PathBuf> {
        let base = xdg::BaseDirectories::new().context("failed to parse XDG_CONFIG_HOME")?;
        Ok(base.get_config_home().join(path))
    }
    #[cfg(windows)]
    fn xdg_prefix(&self, _path: &str) -> Result<PathBuf> {
        Err(anyhow!("{} is not supported on Windows", self.name()))
    }
}

#[derive(Clone, Debug)]
struct XdgDataPrefix;
impl XdgPrefix for XdgDataPrefix {
    fn name(&self) -> &str {
        "xdg_data_prefix"
    }
    fn var(&self) -> &str {
        "XDG_DATA_HOME"
    }
    #[cfg(unix)]
    fn xdg_prefix(&self, path: &str) -> Result<PathBuf> {
        let base = xdg::BaseDirectories::new().context("failed to parse XDG_DATA_HOME")?;
        Ok(base.get_data_home().join(path))
    }
    #[cfg(windows)]
    fn xdg_prefix(&self, _path: &str) -> Result<PathBuf> {
        Err(anyhow!("{} is not supported on Windows", self.name()))
    }
}

#[derive(Clone, Debug)]
struct XdgStatePrefix;
impl XdgPrefix for XdgStatePrefix {
    fn name(&self) -> &str {
        "xdg_state_prefix"
    }
    fn var(&self) -> &str {
        "XDG_STATE_HOME"
    }
    #[cfg(unix)]
    fn xdg_prefix(&self, path: &str) -> Result<PathBuf> {
        let base = xdg::BaseDirectories::new().context("failed to parse XDG_STATE_HOME")?;
        Ok(base.get_state_home().join(path))
    }
    #[cfg(windows)]
    fn xdg_prefix(&self, _path: &str) -> Result<PathBuf> {
        Err(anyhow!("{} is not supported on Windows", self.name()))
    }
}

#[derive(Debug)]
struct XdgPrefixBuilder<T>
where
    T: XdgPrefix + Clone + 'static,
{
    prefix: T,
    path: String,
}

impl<T> builder::Builder for XdgPrefixBuilder<T>
where
    T: XdgPrefix + Clone + 'static,
{
    fn build(&self, state: &mut builder::State) -> Result<Option<Box<dyn Module>>> {
        state.prefix.set(self.prefix.xdg_prefix(&self.path)?);
        Ok(None)
    }
}

#[derive(Clone)]
struct XdgPrefixParser<T>(T)
where
    T: XdgPrefix + Clone + 'static;

impl<T> builder::Parser for XdgPrefixParser<T>
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
    fn parse(&self, args: &[&str]) -> Result<Box<dyn builder::Builder>> {
        let path = util::single_arg(self.0.name(), args)?.to_owned();
        Ok(Box::new(XdgPrefixBuilder {
            prefix: self.0.clone(),
            path,
        }))
    }
}

pub fn commands() -> Vec<Box<dyn builder::Parser>> {
    vec![
        Box::new(XdgPrefixParser(XdgCachePrefix {})),
        Box::new(XdgPrefixParser(XdgConfigPrefix {})),
        Box::new(XdgPrefixParser(XdgDataPrefix {})),
        Box::new(XdgPrefixParser(XdgStatePrefix {})),
    ]
}
