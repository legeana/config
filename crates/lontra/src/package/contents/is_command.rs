use std::path::Path;

use anyhow::{Context as _, Result};
use indoc::formatdoc;

use crate::command::is_command;

use super::args::Arguments;
use super::engine;
use super::inventory;

#[derive(Debug)]
struct IsCommand(String);

impl engine::Condition for IsCommand {
    fn eval(&self, _ctx: &engine::Context) -> Result<bool> {
        is_command(&self.0)
    }
}

#[derive(Clone, Debug)]
struct IsCommandBuilder;

impl engine::ConditionBuilder for IsCommandBuilder {
    fn name(&self) -> String {
        "is_command".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <executable>
                true if <executable> is an executable command
        ", command=self.name()}
    }
    fn build(&self, _workdir: &Path, args: &Arguments) -> Result<engine::BoxedCondition> {
        let exe = args
            .expect_single_arg(self.name())?
            .expect_raw()
            .context("executable")?
            .to_owned();
        Ok(Box::new(IsCommand(exe)))
    }
}

pub(super) fn register(registry: &mut dyn inventory::Registry) {
    registry.register_condition(Box::new(IsCommandBuilder));
}
