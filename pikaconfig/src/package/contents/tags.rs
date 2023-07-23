use std::path::Path;

use anyhow::{Context, Result};
use indoc::formatdoc;

use crate::module::ModuleBox;
use crate::tag_util;

use super::engine;
use super::inventory;
use super::util;

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
                do not process current directory if any of the tags is not present
        ", command=self.name()}
    }
    fn parse(&self, _workdir: &Path, args: &[&str]) -> Result<engine::StatementBox> {
        let (_, tags) = util::multiple_args(&self.name(), args, 0)?;
        Ok(Box::new(RequiresStatement {
            tags: tags.iter().map(|&s| s.to_owned()).collect(),
        }))
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
                do not process current directory if any of the tags is present
        ", command=self.name()}
    }
    fn parse(&self, _workdir: &Path, args: &[&str]) -> Result<engine::StatementBox> {
        let (_, tags) = util::multiple_args(&self.name(), args, 0)?;
        Ok(Box::new(ConflictsStatement {
            tags: tags.iter().map(|&s| s.to_owned()).collect(),
        }))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(RequiresBuilder {}));
    registry.register_command(Box::new(ConflictsBuilder {}));
}
