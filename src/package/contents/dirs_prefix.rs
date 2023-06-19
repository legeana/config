use std::path::PathBuf;

use anyhow::{anyhow, Result};
use indoc::formatdoc;

use crate::handlebars_helper;
use crate::module::Module;
use crate::xdg_or_win;

use super::builder;
use super::builder::Parser;
use super::util;

#[derive(Debug)]
struct DirsPrefixBuilder {
    command: &'static str,
    base_dir: Option<PathBuf>,
    subdir: String,
}

impl builder::Builder for DirsPrefixBuilder {
    fn build(&self, state: &mut builder::State) -> Result<Option<Box<dyn Module>>> {
        let base_dir = self
            .base_dir
            .as_ref()
            .ok_or_else(|| anyhow!("{} is not supported", self.command))?;
        state.prefix = base_dir.join(&self.subdir);
        Ok(None)
    }
}

#[derive(Clone)]
struct DirsPrefixParser {
    command: &'static str,
    base_dir: Option<PathBuf>,
}

impl builder::Parser for DirsPrefixParser {
    fn name(&self) -> String {
        self.command.to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <directory>
                set current installation prefix to {base_dir:?}/<directory>
        ", command=self.name(), base_dir=self.base_dir}
    }
    fn parse(
        &self,
        _workdir: &std::path::Path,
        args: &[&str],
    ) -> Result<Box<dyn builder::Builder>> {
        let subdir = util::single_arg(&self.name(), args)?.to_owned();
        Ok(Box::new(DirsPrefixBuilder {
            command: self.command,
            base_dir: self.base_dir.clone(),
            subdir,
        }))
    }
    fn register_render_helper(&self, hb: &mut handlebars::Handlebars) -> Result<()> {
        if self.base_dir.is_none() {
            return Ok(());
        }
        hb.register_helper(&self.name(), handlebars_helper::wrap(self.clone()));
        Ok(())
    }
}

impl handlebars_helper::SimpleHelper for DirsPrefixParser {
    fn call_inner(&self, params: &[&serde_json::Value]) -> Result<serde_json::Value> {
        if params.len() != 1 {
            return Err(anyhow!(
                "{} requires a single argument, got {}",
                self.name(),
                params.len()
            ));
        }
        let path = params[0].as_str().ok_or_else(|| {
            anyhow!(
                "{} argument must be a string, got {:?}",
                self.name(),
                params[0]
            )
        })?;
        let base_dir = self
            .base_dir
            .as_ref()
            .ok_or_else(|| anyhow!("{} is not supported", self.name()))?;
        let result = base_dir.join(path);
        let result = result
            .to_str()
            .ok_or_else(|| anyhow!("{}: unable to convert {result:?} to string", self.name()))?;
        Ok(result.into())
    }
}

/*impl handlebars::HelperDef for DirsPrefixParser {
    fn call_inner<'reg: 'rc, 'rc>(
            &self,
            _: &handlebars::Helper<'reg, 'rc>,
            _: &'reg handlebars::Handlebars<'reg>,
            _: &'rc handlebars::Context,
            _: &mut handlebars::RenderContext<'reg, 'rc>,
        ) -> std::result::Result<handlebars::ScopedJson<'reg, 'rc>, handlebars::RenderError> {
        let
        Ok(handlebars::ScopedJson::Derived(handlebars::JsonValue::Null))
    }
}*/

pub fn commands() -> Vec<Box<dyn builder::Parser>> {
    vec![
        Box::new(DirsPrefixParser {
            command: "audio_prefix",
            base_dir: dirs::audio_dir(),
        }),
        Box::new(DirsPrefixParser {
            command: "cache_prefix",
            base_dir: dirs::cache_dir(),
        }),
        Box::new(DirsPrefixParser {
            command: "config_prefix",
            base_dir: dirs::config_dir(),
        }),
        Box::new(DirsPrefixParser {
            command: "config_local_prefix",
            base_dir: dirs::config_local_dir(),
        }),
        Box::new(DirsPrefixParser {
            command: "data_prefix",
            base_dir: dirs::data_dir(),
        }),
        Box::new(DirsPrefixParser {
            command: "data_local_prefix",
            base_dir: dirs::data_local_dir(),
        }),
        Box::new(DirsPrefixParser {
            command: "desktop_prefix",
            base_dir: dirs::desktop_dir(),
        }),
        Box::new(DirsPrefixParser {
            command: "document_prefix",
            base_dir: dirs::document_dir(),
        }),
        Box::new(DirsPrefixParser {
            command: "download_prefix",
            base_dir: dirs::download_dir(),
        }),
        Box::new(DirsPrefixParser {
            command: "executable_prefix",
            base_dir: dirs::executable_dir(),
        }),
        Box::new(DirsPrefixParser {
            command: "font_prefix",
            base_dir: dirs::font_dir(),
        }),
        Box::new(DirsPrefixParser {
            command: "home_prefix",
            base_dir: dirs::home_dir(),
        }),
        Box::new(DirsPrefixParser {
            command: "picture_prefix",
            base_dir: dirs::picture_dir(),
        }),
        Box::new(DirsPrefixParser {
            command: "preference_prefix",
            base_dir: dirs::preference_dir(),
        }),
        Box::new(DirsPrefixParser {
            command: "public_prefix",
            base_dir: dirs::public_dir(),
        }),
        Box::new(DirsPrefixParser {
            command: "runtime_prefix",
            base_dir: dirs::runtime_dir(),
        }),
        Box::new(DirsPrefixParser {
            command: "state_prefix",
            base_dir: dirs::state_dir(),
        }),
        Box::new(DirsPrefixParser {
            command: "template_prefix",
            base_dir: dirs::template_dir(),
        }),
        Box::new(DirsPrefixParser {
            command: "video_prefix",
            base_dir: dirs::video_dir(),
        }),
        // XDG (for UNIX) or Windows.
        Box::new(DirsPrefixParser {
            command: "xdg_or_win_cache_prefix",
            base_dir: xdg_or_win::cache_dir(),
        }),
        Box::new(DirsPrefixParser {
            command: "xdg_or_win_config_prefix",
            base_dir: xdg_or_win::config_dir(),
        }),
        Box::new(DirsPrefixParser {
            command: "xdg_or_win_config_local_prefix",
            base_dir: xdg_or_win::config_local_dir(),
        }),
        Box::new(DirsPrefixParser {
            command: "xdg_or_win_data_prefix",
            base_dir: xdg_or_win::data_dir(),
        }),
        Box::new(DirsPrefixParser {
            command: "xdg_or_win_data_local_prefix",
            base_dir: xdg_or_win::data_local_dir(),
        }),
    ]
}
