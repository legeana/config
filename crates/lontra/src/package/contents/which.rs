use std::path::Path;
use std::sync::Arc;

use anyhow::{Context as _, Result};
use indoc::formatdoc;
use minijinja::Environment;

use super::args::{Argument, Arguments};
use super::engine;
use super::engine::CommandBuilder as _;
use super::inventory;
use crate::jinja;

#[derive(Debug)]
struct WhichExpression {
    binary: Argument,
}

impl engine::Expression for WhichExpression {
    fn eval(&self, ctx: &mut engine::Context) -> Result<engine::ExpressionOutput> {
        let binary = ctx.expand_arg(&self.binary).context("binary")?;
        let output = which::which(&binary)
            .with_context(|| format!("failed to find {binary:?} path"))?
            .into_os_string();
        Ok(engine::ExpressionOutput {
            module: None,
            // TODO: The output is evaluated eagerly during the MANIFEST parsing
            // and will only pick up binaries already available in the path.
            output,
        })
    }
}

#[derive(Clone)]
struct WhichBuilder;

impl engine::CommandBuilder for WhichBuilder {
    fn name(&self) -> String {
        "which".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            <path> = $({command} <binary>)
                Returns a full path to a binary in $PATH
        ", command=self.name()}
    }
    fn build(&self, _workdir: &Path, args: &Arguments) -> Result<engine::Command> {
        let binary = args.expect_single_arg(self.name())?;
        Ok(engine::Command::new_expression(WhichExpression {
            binary: binary.clone(),
        }))
    }
}

impl inventory::RenderHelper for WhichBuilder {
    fn register_globals(&self, env: &mut Environment, _ctx: &Arc<jinja::Context>) {
        use crate::jinja::{JResult, map_anyhow, map_error, to_string};
        env.add_function(self.name(), |binary: &str| -> JResult<String> {
            let path = which::which(binary)
                .with_context(|| format!("failed to find {binary:?} path"))
                .map_err(map_anyhow)?;
            to_string("path", path).map_err(map_error)
        });
    }
}

pub(super) fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(WhichBuilder));
    registry.register_render_helper(Box::new(WhichBuilder));
}
