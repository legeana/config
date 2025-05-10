use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context as _, Result, anyhow};
use indoc::formatdoc;
use lontra_xdg as xdg;
use lontra_xdg::xdg_or_win;
use minijinja::Environment;

use crate::jinja;
use crate::module::BoxedModule;

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
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<BoxedModule>> {
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

#[derive(Clone)]
struct DirsPrefixBuilder {
    manifest_command: &'static str,
    render_command: &'static str,
    base_dir: Option<PathBuf>,
}

impl engine::CommandBuilder for DirsPrefixBuilder {
    fn name(&self) -> String {
        self.manifest_command.to_owned()
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
            command: self.manifest_command,
            base_dir: self.base_dir.clone(),
            subdir,
        }))
    }
}

impl inventory::RenderHelper for DirsPrefixBuilder {
    fn register_globals(&self, env: &mut Environment, _ctx: &Arc<jinja::Context>) {
        use crate::jinja::{JResult, map_error, to_string};
        let Some(base_dir) = self.base_dir.clone() else {
            return;
        };
        env.add_function(
            self.render_command,
            move |path: Option<String>| -> JResult<String> {
                let joined = match &path {
                    Some(path) => base_dir.join(path),
                    None => base_dir.clone(),
                };
                to_string("path", joined).map_err(map_error)
            },
        );
    }
}

pub(super) fn register(registry: &mut dyn inventory::Registry) {
    let parsers = [
        DirsPrefixBuilder {
            manifest_command: "audio_prefix",
            render_command: "audio_dir",
            base_dir: dirs::audio_dir(),
        },
        DirsPrefixBuilder {
            manifest_command: "cache_prefix",
            render_command: "cache_dir",
            base_dir: dirs::cache_dir(),
        },
        DirsPrefixBuilder {
            manifest_command: "config_prefix",
            render_command: "config_dir",
            base_dir: dirs::config_dir(),
        },
        DirsPrefixBuilder {
            manifest_command: "config_local_prefix",
            render_command: "config_local_dir",
            base_dir: dirs::config_local_dir(),
        },
        DirsPrefixBuilder {
            manifest_command: "data_prefix",
            render_command: "data_dir",
            base_dir: dirs::data_dir(),
        },
        DirsPrefixBuilder {
            manifest_command: "data_local_prefix",
            render_command: "data_local_dir",
            base_dir: dirs::data_local_dir(),
        },
        DirsPrefixBuilder {
            manifest_command: "desktop_prefix",
            render_command: "desktop_dir",
            base_dir: dirs::desktop_dir(),
        },
        DirsPrefixBuilder {
            manifest_command: "document_prefix",
            render_command: "document_dir",
            base_dir: dirs::document_dir(),
        },
        DirsPrefixBuilder {
            manifest_command: "download_prefix",
            render_command: "download_dir",
            base_dir: dirs::download_dir(),
        },
        DirsPrefixBuilder {
            manifest_command: "executable_prefix",
            render_command: "executable_dir",
            base_dir: dirs::executable_dir(),
        },
        DirsPrefixBuilder {
            manifest_command: "font_prefix",
            render_command: "font_dir",
            base_dir: dirs::font_dir(),
        },
        DirsPrefixBuilder {
            manifest_command: "home_prefix",
            render_command: "home_dir",
            base_dir: dirs::home_dir(),
        },
        DirsPrefixBuilder {
            manifest_command: "picture_prefix",
            render_command: "picture_dir",
            base_dir: dirs::picture_dir(),
        },
        DirsPrefixBuilder {
            manifest_command: "preference_prefix",
            render_command: "preference_dir",
            base_dir: dirs::preference_dir(),
        },
        DirsPrefixBuilder {
            manifest_command: "public_prefix",
            render_command: "public_dir",
            base_dir: dirs::public_dir(),
        },
        DirsPrefixBuilder {
            manifest_command: "runtime_prefix",
            render_command: "runtime_dir",
            base_dir: dirs::runtime_dir(),
        },
        DirsPrefixBuilder {
            manifest_command: "state_prefix",
            render_command: "state_dir",
            base_dir: dirs::state_dir(),
        },
        DirsPrefixBuilder {
            manifest_command: "template_prefix",
            render_command: "template_dir",
            base_dir: dirs::template_dir(),
        },
        DirsPrefixBuilder {
            manifest_command: "video_prefix",
            render_command: "video_dir",
            base_dir: dirs::video_dir(),
        },
        // XDG
        DirsPrefixBuilder {
            manifest_command: "xdg_cache_prefix",
            render_command: "xdg_cache_dir",
            base_dir: xdg::cache_dir(),
        },
        DirsPrefixBuilder {
            manifest_command: "xdg_config_prefix",
            render_command: "xdg_config_dir",
            base_dir: xdg::config_dir(),
        },
        DirsPrefixBuilder {
            manifest_command: "xdg_data_prefix",
            render_command: "xdg_data_dir",
            base_dir: xdg::data_dir(),
        },
        DirsPrefixBuilder {
            manifest_command: "xdg_executable_prefix",
            render_command: "xdg_executable_dir",
            base_dir: xdg::executable_dir(),
        },
        DirsPrefixBuilder {
            manifest_command: "xdg_state_prefix",
            render_command: "xdg_state_dir",
            base_dir: xdg::state_dir(),
        },
        // XDG (for UNIX) or Windows.
        DirsPrefixBuilder {
            manifest_command: "xdg_or_win_cache_prefix",
            render_command: "xdg_or_win_cache_dir",
            base_dir: xdg_or_win::cache_dir(),
        },
        DirsPrefixBuilder {
            manifest_command: "xdg_or_win_config_prefix",
            render_command: "xdg_or_win_config_dir",
            base_dir: xdg_or_win::config_dir(),
        },
        DirsPrefixBuilder {
            manifest_command: "xdg_or_win_config_local_prefix",
            render_command: "xdg_or_win_config_local_dir",
            base_dir: xdg_or_win::config_local_dir(),
        },
        DirsPrefixBuilder {
            manifest_command: "xdg_or_win_data_prefix",
            render_command: "xdg_or_win_data_dir",
            base_dir: xdg_or_win::data_dir(),
        },
        DirsPrefixBuilder {
            manifest_command: "xdg_or_win_data_local_prefix",
            render_command: "xdg_or_win_data_local_dir",
            base_dir: xdg_or_win::data_local_dir(),
        },
    ];
    for dir in parsers {
        registry.register_command(Box::new(dir.clone()));
        registry.register_render_helper(Box::new(dir));
    }
}
