use anyhow::Context;

use super::parser;
use super::util;
use crate::tag_util;

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
    fn parse(
        &self,
        _state: &mut parser::State,
        configuration: &mut super::Configuration,
        args: &[&str],
    ) -> parser::Result<()> {
        let (_, tags) = util::multiple_args(REQUIRES_COMMAND, args, 0)?;
        for tag in tags.iter() {
            let has_tag =
                tag_util::has_tag(tag).with_context(|| format!("failed to check tag {tag}"))?;
            if !has_tag {
                configuration.enabled = false;
            }
        }
        Ok(())
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
    fn parse(
        &self,
        _state: &mut parser::State,
        configuration: &mut super::Configuration,
        args: &[&str],
    ) -> parser::Result<()> {
        let (_, tags) = util::multiple_args(CONFLICTS_COMMAND, args, 0)?;
        for tag in tags.iter() {
            let has_tag =
                tag_util::has_tag(tag).with_context(|| format!("failed to check tag {tag}"))?;
            if has_tag {
                configuration.enabled = false;
            }
        }
        Ok(())
    }
}
