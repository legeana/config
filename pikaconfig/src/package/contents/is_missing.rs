use std::path::Path;

use super::args::{Argument, Arguments};
use super::engine;
use super::inventory;

use anyhow::{Context, Result};
use indoc::formatdoc;

#[derive(Debug)]
struct IsMissing(Argument);

impl engine::Condition for IsMissing {
    fn eval(&self, ctx: &engine::Context) -> Result<bool> {
        let path = ctx.dst_path(ctx.expand_arg(&self.0)?);
        path.try_exists()
            .with_context(|| format!("failed to check if {path:?} exists"))
    }
}

#[derive(Clone)]
struct IsMissingBuilder;

impl engine::ConditionBuilder for IsMissingBuilder {
    fn name(&self) -> String {
        "is_missing".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <path>
                true if <path> does not exist
        ", command=self.name()}
    }
    fn build(&self, _workdir: &Path, args: &Arguments) -> Result<engine::ConditionBox> {
        let path = args.expect_single_arg(self.name())?.clone();
        Ok(Box::new(IsMissing(path)))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_condition(Box::new(IsMissingBuilder));
}
