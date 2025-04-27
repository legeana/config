use std::path::Path;

use super::args::{Argument, Arguments};
use super::engine;
use super::inventory;

use anyhow::{Context as _, Result};
use indoc::formatdoc;

#[derive(Debug)]
struct Exists(Argument);

impl engine::Condition for Exists {
    fn eval(&self, ctx: &engine::Context) -> Result<bool> {
        let path = ctx.dst_path(ctx.expand_arg(&self.0)?);
        path.try_exists()
            .with_context(|| format!("failed to check if {path:?} exists"))
    }
}

#[derive(Clone)]
struct ExistsBuilder;

impl engine::ConditionBuilder for ExistsBuilder {
    fn name(&self) -> String {
        "exists".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <path>
                true if <path> exists
        ", command=self.name()}
    }
    fn build(&self, _workdir: &Path, args: &Arguments) -> Result<engine::BoxedCondition> {
        let path = args.expect_single_arg(self.name())?.clone();
        Ok(Box::new(Exists(path)))
    }
}

pub(super) fn register(registry: &mut dyn inventory::Registry) {
    registry.register_condition(Box::new(ExistsBuilder));
}
