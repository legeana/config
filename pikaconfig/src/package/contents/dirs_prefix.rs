use std::path::PathBuf;

use anyhow::{anyhow, Result};
use indoc::formatdoc;
use serde::Deserialize;

use crate::module::ModuleBox;
use crate::tera_helper;
use crate::xdg;
use crate::xdg_or_win;

use super::args::Arguments;
use super::engine;
use super::inventory;
use super::util;

#[derive(Debug)]
struct DirsPrefixStatement {
    command: &'static str,
    base_dir: Option<PathBuf>,
    subdir: String,
}

impl engine::Statement for DirsPrefixStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        let base_dir = self
            .base_dir
            .as_ref()
            .ok_or_else(|| anyhow!("{} is not supported", self.command))?;
        ctx.prefix = base_dir.join(&self.subdir);
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
        formatdoc! {"
            {command} <directory>
                set current installation prefix to {base_dir:?}/<directory>
        ", command=self.name(), base_dir=self.base_dir}
    }
    fn build(&self, _workdir: &std::path::Path, args: &Arguments) -> Result<engine::StatementBox> {
        let subdir = util::single_arg(&self.name(), args)?.to_owned();
        Ok(Box::new(DirsPrefixStatement {
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

pub fn register(registry: &mut dyn inventory::Registry) {
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
