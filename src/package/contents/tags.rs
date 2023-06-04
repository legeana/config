use anyhow::{Context, Result};

use crate::module::Module;
use crate::tag_util;

use super::builder;
use super::util;

pub struct RequiresBuilder;
pub struct ConflictsBuilder;

const REQUIRES_COMMAND: &str = "requires";
const CONFLICTS_COMMAND: &str = "conflicts";

impl builder::Builder for RequiresBuilder {
    fn name(&self) -> String {
        REQUIRES_COMMAND.to_owned()
    }
    fn help(&self) -> String {
        format!("{REQUIRES_COMMAND} <tags>
           do not process current directory if any of the tags is not present")
    }
    fn build(&self, state: &mut builder::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        let (_, tags) = util::multiple_args(REQUIRES_COMMAND, args, 0)?;
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

impl builder::Builder for ConflictsBuilder {
    fn name(&self) -> String {
        CONFLICTS_COMMAND.to_owned()
    }
    fn help(&self) -> String {
        format!("{CONFLICTS_COMMAND} <tags>
           do not process current directory if any of the tags is present")
    }
    fn build(&self, state: &mut builder::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        let (_, tags) = util::multiple_args(CONFLICTS_COMMAND, args, 0)?;
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
