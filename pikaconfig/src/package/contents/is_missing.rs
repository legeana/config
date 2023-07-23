use std::path::Path;
use std::path::PathBuf;

use super::args::Arguments;
use super::engine;
use super::inventory;

use anyhow::{Context, Result};
use indoc::formatdoc;

#[derive(Debug)]
struct IsMissing(PathBuf);

impl engine::Condition for IsMissing {
    fn eval(&self, _ctx: &engine::Context) -> Result<bool> {
        // TODO: should this depend on prefix?
        self.0
            .try_exists()
            .with_context(|| format!("failed to check if {:?} exists", &self.0))
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
        let path = args.expect_single_arg(self.name())?;
        Ok(Box::new(IsMissing(path.into())))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_condition(Box::new(IsMissingBuilder));
}
