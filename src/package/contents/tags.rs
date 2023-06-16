use anyhow::{Context, Result};
use indoc::formatdoc;

use crate::module::Module;
use crate::tag_util;

use super::builder;
use super::util;

struct RequiresBuilder;

impl builder::Builder for RequiresBuilder {
    fn name(&self) -> String {
        "requires".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <tags>
                do not process current directory if any of the tags is not present
        ", command=self.name()}
    }
    fn build(&self, state: &mut builder::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        let (_, tags) = util::multiple_args(&self.name(), args, 0)?;
        for tag in tags.iter() {
            let has_tag =
                tag_util::has_tag(tag).with_context(|| format!("failed to check tag {tag}"))?;
            if !has_tag {
                state.enabled = false;
            }
        }
        Ok(None)
    }
}

struct ConflictsBuilder;

impl builder::Builder for ConflictsBuilder {
    fn name(&self) -> String {
        "conflicts".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <tags>
                do not process current directory if any of the tags is present
        ", command=self.name()}
    }
    fn build(&self, state: &mut builder::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        let (_, tags) = util::multiple_args(&self.name(), args, 0)?;
        for tag in tags.iter() {
            let has_tag =
                tag_util::has_tag(tag).with_context(|| format!("failed to check tag {tag}"))?;
            if has_tag {
                state.enabled = false;
            }
        }
        Ok(None)
    }
}

pub fn commands() -> Vec<Box<dyn builder::Builder>> {
    vec![Box::new(RequiresBuilder {}), Box::new(ConflictsBuilder {})]
}
