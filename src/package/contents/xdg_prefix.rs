use super::builder;

#[cfg(unix)]
mod unix_only {
    use std::path::PathBuf;

    use anyhow::{Context, Result};
    use indoc::formatdoc;

    use crate::module::Module;

    use crate::package::contents::builder;
    use crate::package::contents::util;

    trait XdgPrefix {
        fn name(&self) -> &str;
        fn var(&self) -> &str;
        fn xdg_prefix(&self, path: &str) -> Result<PathBuf>;
    }

    struct XdgCachePrefixBuilder;
    impl XdgPrefix for XdgCachePrefixBuilder {
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

    struct XdgConfigPrefixBuilder;
    impl XdgPrefix for XdgConfigPrefixBuilder {
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

    struct XdgDataPrefixBuilder;
    impl XdgPrefix for XdgDataPrefixBuilder {
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

    struct XdgStatePrefixBuilder;
    impl XdgPrefix for XdgStatePrefixBuilder {
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

    struct XdgPrefixBuilder<T>(T)
    where
        T: XdgPrefix;

    impl<T> builder::Builder for XdgPrefixBuilder<T>
    where
        T: XdgPrefix,
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
        fn build(
            &self,
            state: &mut builder::State,
            args: &[&str],
        ) -> Result<Option<Box<dyn Module>>> {
            let path = util::single_arg(self.0.name(), args)?;
            state.prefix.set(self.0.xdg_prefix(path)?);
            Ok(None)
        }
    }
    pub fn commands() -> Vec<Box<dyn builder::Builder>> {
        vec![
            Box::new(XdgPrefixBuilder(XdgCachePrefixBuilder {})),
            Box::new(XdgPrefixBuilder(XdgConfigPrefixBuilder {})),
            Box::new(XdgPrefixBuilder(XdgDataPrefixBuilder {})),
            Box::new(XdgPrefixBuilder(XdgStatePrefixBuilder {})),
        ]
    }
}

#[cfg(unix)]
pub fn commands() -> Vec<Box<dyn builder::Builder>> {
    unix_only::commands()
}

#[cfg(windows)]
pub fn commands() -> Vec<Box<dyn builder::Builder>> {
    Vec::default()
}
