use std::path::Path;

use anyhow::Result;
use indoc::formatdoc;

use crate::module::ModuleBox;

use super::args::{Argument, Arguments};
use super::engine;
use super::inventory;

#[derive(Debug)]
struct PrefixStatement {
    prefix: Argument,
}

impl engine::Statement for PrefixStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        ctx.prefix = ctx.expand_arg(&self.prefix)?.into();
        Ok(None)
    }
}

#[derive(Clone)]
struct PrefixBuilder;

impl engine::CommandBuilder for PrefixBuilder {
    fn name(&self) -> String {
        "prefix".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <directory>
                set current installation prefix to <directory>
        ", command=self.name()}
    }
    fn build(&self, _workdir: &Path, args: &Arguments) -> Result<engine::Command> {
        let prefix = args.expect_single_arg(self.name())?.clone();
        Ok(engine::Command::new_statement(PrefixStatement { prefix }))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(PrefixBuilder));
}
