use std::path::PathBuf;

use anyhow::{Context as _, Result, anyhow};
use indoc::formatdoc;
use serde::Deserialize;
use xdg::xdg_or_win;

use crate::module::ModuleBox;
use crate::tera_helper;

use super::args::Argument;
use super::args::Arguments;
use super::engine;
use super::inventory;

#[derive(Debug)]
struct DirsPrefixStatement {
    command: &'static str,
    base_dir: Option<PathBuf>,
    subdir: Option<String>,
}

impl engine::Statement for DirsPrefixStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        let base_dir = self
            .base_dir
            .as_ref()
            .ok_or_else(|| anyhow!("{} is not supported", self.command))?;
        ctx.prefix = match &self.subdir {
            Some(s) => base_dir.join(s),
            None => base_dir.clone(),
        };
        Ok(None)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DirsPrefixParams {
    path: Option<PathBuf>,
}

#[derive(Clone)]
struct DirsPrefixBuilder {
    command: &'static str,
    base_dir: Option<PathBuf>,
}

impl engine::CommandBuilder for DirsPrefixBuilder {
    fn name(&self) -> String {
        self.command.to_owned()
    }
    fn help(&self) -> String {
        let prefix = self
            .base_dir
            .as_ref()
            .map(|base| base.join("<directory>"))
            .unwrap_or("<unavailable>".into());
        formatdoc! {"
            {command} [<directory>]
                set current installation prefix to {prefix:?}
        ", command=self.name(), prefix=prefix}
    }
    fn build(&self, _workdir: &std::path::Path, args: &Arguments) -> Result<engine::Command> {
        let subdir = args
            .expect_optional_arg(self.name())?
            .map(Argument::expect_raw)
            .transpose()
            .context("subdir")?
            .map(ToOwned::to_owned);
        Ok(engine::Command::new_statement(DirsPrefixStatement {
            command: self.command,
            base_dir: self.base_dir.clone(),
            subdir,
        }))
    }
}

impl inventory::RenderHelper for DirsPrefixBuilder {
    fn register_render_helper(&self, tera: &mut tera::Tera) {
        let Some(base_dir) = self.base_dir.clone() else {
            return;
        };
        tera.register_function(
            &engine::CommandBuilder::name(self),
            tera_helper::wrap_fn(move |args: &DirsPrefixParams| {
                Ok(match args.path {
                    Some(ref path) => base_dir.join(path),
                    None => base_dir.clone(),
                })
            }),
        );
    }
}

pub(super) fn register(registry: &mut dyn inventory::Registry) {
    let parsers = [
        DirsPrefixBuilder {
            command: "audio_prefix",
            base_dir: dirs::audio_dir(),
        },
        DirsPrefixBuilder {
            command: "cache_prefix",
            base_dir: dirs::cache_dir(),
        },
        DirsPrefixBuilder {
            command: "config_prefix",
            base_dir: dirs::config_dir(),
        },
        DirsPrefixBuilder {
            command: "config_local_prefix",
            base_dir: dirs::config_local_dir(),
        },
        DirsPrefixBuilder {
            command: "data_prefix",
            base_dir: dirs::data_dir(),
        },
        DirsPrefixBuilder {
            command: "data_local_prefix",
            base_dir: dirs::data_local_dir(),
        },
        DirsPrefixBuilder {
            command: "desktop_prefix",
            base_dir: dirs::desktop_dir(),
        },
        DirsPrefixBuilder {
            command: "document_prefix",
            base_dir: dirs::document_dir(),
        },
        DirsPrefixBuilder {
            command: "download_prefix",
            base_dir: dirs::download_dir(),
        },
        DirsPrefixBuilder {
            command: "executable_prefix",
            base_dir: dirs::executable_dir(),
        },
        DirsPrefixBuilder {
            command: "font_prefix",
            base_dir: dirs::font_dir(),
        },
        DirsPrefixBuilder {
            command: "home_prefix",
            base_dir: dirs::home_dir(),
        },
        DirsPrefixBuilder {
            command: "picture_prefix",
            base_dir: dirs::picture_dir(),
        },
        DirsPrefixBuilder {
            command: "preference_prefix",
            base_dir: dirs::preference_dir(),
        },
        DirsPrefixBuilder {
            command: "public_prefix",
            base_dir: dirs::public_dir(),
        },
        DirsPrefixBuilder {
            command: "runtime_prefix",
            base_dir: dirs::runtime_dir(),
        },
        DirsPrefixBuilder {
            command: "state_prefix",
            base_dir: dirs::state_dir(),
        },
        DirsPrefixBuilder {
            command: "template_prefix",
            base_dir: dirs::template_dir(),
        },
        DirsPrefixBuilder {
            command: "video_prefix",
            base_dir: dirs::video_dir(),
        },
        // XDG
        DirsPrefixBuilder {
            command: "xdg_cache_prefix",
            base_dir: xdg::cache_dir(),
        },
        DirsPrefixBuilder {
            command: "xdg_config_prefix",
            base_dir: xdg::config_dir(),
        },
        DirsPrefixBuilder {
            command: "xdg_data_prefix",
            base_dir: xdg::data_dir(),
        },
        DirsPrefixBuilder {
            command: "xdg_executable_prefix",
            base_dir: xdg::executable_dir(),
        },
        DirsPrefixBuilder {
            command: "xdg_state_prefix",
            base_dir: xdg::state_dir(),
        },
        // XDG (for UNIX) or Windows.
        DirsPrefixBuilder {
            command: "xdg_or_win_cache_prefix",
            base_dir: xdg_or_win::cache_dir(),
        },
        DirsPrefixBuilder {
            command: "xdg_or_win_config_prefix",
            base_dir: xdg_or_win::config_dir(),
        },
        DirsPrefixBuilder {
            command: "xdg_or_win_config_local_prefix",
            base_dir: xdg_or_win::config_local_dir(),
        },
        DirsPrefixBuilder {
            command: "xdg_or_win_data_prefix",
            base_dir: xdg_or_win::data_dir(),
        },
        DirsPrefixBuilder {
            command: "xdg_or_win_data_local_prefix",
            base_dir: xdg_or_win::data_local_dir(),
        },
    ];
    for dir in parsers {
        registry.register_command(Box::new(dir.clone()));
        registry.register_render_helper(Box::new(dir));
    }
}
