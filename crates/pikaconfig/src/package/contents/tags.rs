use std::path::Path;

use anyhow::{Context as _, Result};
use indoc::formatdoc;

use super::args::Arguments;
use super::engine;
use super::inventory;

#[derive(Debug)]
struct TagsCondition(Vec<String>);

impl engine::Condition for TagsCondition {
    fn eval(&self, _ctx: &engine::Context) -> Result<bool> {
        for tag in &self.0 {
            let has_tag =
                tag_util::has_tag(tag).with_context(|| format!("failed to check tag {tag}"))?;
            if !has_tag {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

#[derive(Clone)]
struct TagsBuilder;

impl engine::ConditionBuilder for TagsBuilder {
    fn name(&self) -> String {
        "tags".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} [<tag>...]
                Returns true if all tags are matching.
                Both tag=value and tag!=value are supported.
        ", command=self.name()}
    }
    fn build(&self, _workdir: &Path, args: &Arguments) -> Result<engine::ConditionBox> {
        let tags = args.expect_any_args(self.name())?;
        let tags: Vec<_> = tags
            .iter()
            .map(|t| t.expect_raw().context("tag").map(str::to_string))
            .collect::<Result<_>>()?;
        Ok(Box::new(TagsCondition(tags)))
    }
}

pub(super) fn register(registry: &mut dyn inventory::Registry) {
    registry.register_condition(Box::new(TagsBuilder));
}
