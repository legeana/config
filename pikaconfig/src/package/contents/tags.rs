use std::path::Path;

use anyhow::{Context, Result};
use indoc::formatdoc;

use crate::module::ModuleBox;
use crate::tag_util;

use super::args::Arguments;
use super::engine;
use super::inventory;

#[derive(Debug)]
struct RequiresStatement {
    tags: Vec<String>,
}

impl engine::Statement for RequiresStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        for tag in self.tags.iter() {
            let has_tag =
                tag_util::has_tag(tag).with_context(|| format!("failed to check tag {tag}"))?;
            if !has_tag {
                ctx.enabled = false;
            }
        }
        Ok(None)
    }
}

#[derive(Clone)]
struct RequiresBuilder;

impl engine::CommandBuilder for RequiresBuilder {
    fn name(&self) -> String {
        "requires".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <tags>
                only process the current directory if all of the tags are present
        ", command=self.name()}
    }
    fn build(&self, _workdir: &Path, args: &Arguments) -> Result<engine::Command> {
        let tags = args.expect_any_args(self.name())?;
        let tags: Vec<_> = tags
            .iter()
            .map(|t| t.expect_raw().context("tag").map(str::to_string))
            .collect::<Result<_>>()?;
        Ok(engine::Command::new_statement(RequiresStatement { tags }))
    }
}

#[derive(Debug)]
struct ConflictsStatement {
    tags: Vec<String>,
}

impl engine::Statement for ConflictsStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        for tag in self.tags.iter() {
            let has_tag =
                tag_util::has_tag(tag).with_context(|| format!("failed to check tag {tag}"))?;
            if has_tag {
                ctx.enabled = false;
            }
        }
        Ok(None)
    }
}

#[derive(Clone)]
struct ConflictsBuilder;

impl engine::CommandBuilder for ConflictsBuilder {
    fn name(&self) -> String {
        "conflicts".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <tags>
                only process the current directory if none of the tags are present
        ", command=self.name()}
    }
    fn build(&self, _workdir: &Path, args: &Arguments) -> Result<engine::Command> {
        let tags = args.expect_any_args(self.name())?;
        let tags: Vec<_> = tags
            .iter()
            .map(|t| t.expect_raw().context("tag").map(str::to_string))
            .collect::<Result<_>>()?;
        Ok(engine::Command::new_statement(ConflictsStatement { tags }))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(RequiresBuilder {}));
    registry.register_command(Box::new(ConflictsBuilder {}));
}
