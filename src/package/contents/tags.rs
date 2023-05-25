use anyhow::{Context, Result};

use crate::module::Module;
use crate::tag_util;

use super::parser;
use super::util;

pub struct RequiresParser;
pub struct ConflictsParser;

const REQUIRES_COMMAND: &str = "requires";
const CONFLICTS_COMMAND: &str = "conflicts";

impl parser::Parser for RequiresParser {
    fn name(&self) -> &'static str {
        REQUIRES_COMMAND
    }
    fn help(&self) -> &'static str {
        "requires <tags>
           do not process current directory if any of the tags is not present"
    }
    fn parse(&self, state: &mut parser::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
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

impl parser::Parser for ConflictsParser {
    fn name(&self) -> &'static str {
        CONFLICTS_COMMAND
    }
    fn help(&self) -> &'static str {
        "conflicts <tags>
           do not process current directory if any of the tags is present"
    }
    fn parse(&self, state: &mut parser::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
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
