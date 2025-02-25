use std::path::Path;

use anyhow::Result;
use indoc::formatdoc;

use crate::module::ModuleBox;

use super::args::Arguments;
use super::engine;
use super::inventory;

#[derive(Debug)]
struct ReturnStatement;

impl engine::Statement for ReturnStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        ctx.enabled = false;
        Ok(None)
    }
}

#[derive(Clone)]
struct ReturnBuilder;

impl engine::CommandBuilder for ReturnBuilder {
    fn name(&self) -> String {
        "return".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command}
                stop processing current MANIFEST immediately
        ", command=self.name()}
    }
    fn build(&self, _workdir: &Path, args: &Arguments) -> Result<engine::Command> {
        args.expect_no_args(self.name())?;
        Ok(engine::Command::new_statement(ReturnStatement))
    }
}

pub(super) fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(ReturnBuilder));
}
