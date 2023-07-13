use std::path::PathBuf;

use anyhow::{anyhow, Result};
use indoc::formatdoc;
use serde::Deserialize;

use crate::module::ModuleBox;
use crate::tera_helper;
use crate::xdg;
use crate::xdg_or_win;

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
struct DirsPrefixParser {
    command: &'static str,
    base_dir: Option<PathBuf>,
}

impl engine::Parser for DirsPrefixParser {
    fn name(&self) -> String {
        self.command.to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <directory>
                set current installation prefix to {base_dir:?}/<directory>
        ", command=self.name(), base_dir=self.base_dir}
    }
    fn parse(&self, _workdir: &std::path::Path, args: &[&str]) -> Result<engine::StatementBox> {
        let subdir = util::single_arg(&self.name(), args)?.to_owned();
        Ok(Box::new(DirsPrefixStatement {
            command: self.command,
            base_dir: self.base_dir.clone(),
            subdir,
        }))
    }
}

impl inventory::RenderHelper for DirsPrefixParser {
    fn register_render_helper(&self, tera: &mut tera::Tera) {
        let Some(base_dir) = self.base_dir.clone() else {
            return;
        };
        tera.register_function(
            &engine::Parser::name(self),
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
        DirsPrefixParser {
            command: "audio_prefix",
            base_dir: dirs::audio_dir(),
        },
        DirsPrefixParser {
            command: "cache_prefix",
            base_dir: dirs::cache_dir(),
        },
        DirsPrefixParser {
            command: "config_prefix",
            base_dir: dirs::config_dir(),
        },
        DirsPrefixParser {
            command: "config_local_prefix",
            base_dir: dirs::config_local_dir(),
        },
        DirsPrefixParser {
            command: "data_prefix",
            base_dir: dirs::data_dir(),
        },
        DirsPrefixParser {
            command: "data_local_prefix",
            base_dir: dirs::data_local_dir(),
        },
        DirsPrefixParser {
            command: "desktop_prefix",
            base_dir: dirs::desktop_dir(),
        },
        DirsPrefixParser {
            command: "document_prefix",
            base_dir: dirs::document_dir(),
        },
        DirsPrefixParser {
            command: "download_prefix",
            base_dir: dirs::download_dir(),
        },
        DirsPrefixParser {
            command: "executable_prefix",
            base_dir: dirs::executable_dir(),
        },
        DirsPrefixParser {
            command: "font_prefix",
            base_dir: dirs::font_dir(),
        },
        DirsPrefixParser {
            command: "home_prefix",
            base_dir: dirs::home_dir(),
        },
        DirsPrefixParser {
            command: "picture_prefix",
            base_dir: dirs::picture_dir(),
        },
        DirsPrefixParser {
            command: "preference_prefix",
            base_dir: dirs::preference_dir(),
        },
        DirsPrefixParser {
            command: "public_prefix",
            base_dir: dirs::public_dir(),
        },
        DirsPrefixParser {
            command: "runtime_prefix",
            base_dir: dirs::runtime_dir(),
        },
        DirsPrefixParser {
            command: "state_prefix",
            base_dir: dirs::state_dir(),
        },
        DirsPrefixParser {
            command: "template_prefix",
            base_dir: dirs::template_dir(),
        },
        DirsPrefixParser {
            command: "video_prefix",
            base_dir: dirs::video_dir(),
        },
        // XDG
        DirsPrefixParser {
            command: "xdg_cache_prefix",
            base_dir: xdg::cache_dir(),
        },
        DirsPrefixParser {
            command: "xdg_config_prefix",
            base_dir: xdg::config_dir(),
        },
        DirsPrefixParser {
            command: "xdg_data_prefix",
            base_dir: xdg::data_dir(),
        },
        DirsPrefixParser {
            command: "xdg_state_prefix",
            base_dir: xdg::state_dir(),
        },
        // XDG (for UNIX) or Windows.
        DirsPrefixParser {
            command: "xdg_or_win_cache_prefix",
            base_dir: xdg_or_win::cache_dir(),
        },
        DirsPrefixParser {
            command: "xdg_or_win_config_prefix",
            base_dir: xdg_or_win::config_dir(),
        },
        DirsPrefixParser {
            command: "xdg_or_win_config_local_prefix",
            base_dir: xdg_or_win::config_local_dir(),
        },
        DirsPrefixParser {
            command: "xdg_or_win_data_prefix",
            base_dir: xdg_or_win::data_dir(),
        },
        DirsPrefixParser {
            command: "xdg_or_win_data_local_prefix",
            base_dir: xdg_or_win::data_local_dir(),
        },
    ];
    for dir in parsers {
        registry.register_parser(Box::new(dir.clone()));
        registry.register_render_helper(Box::new(dir));
    }
}
