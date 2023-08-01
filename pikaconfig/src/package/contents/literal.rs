use std::ffi::OsString;
use std::path::Path;

use anyhow::Result;
use indoc::formatdoc;

use super::args::Arguments;
use super::engine;
use super::inventory;

#[derive(Debug)]
struct LiteralExpression(OsString);

impl engine::Expression for LiteralExpression {
    fn eval(&self, _ctx: &mut engine::Context) -> Result<engine::ExpressionOutput> {
        Ok(engine::ExpressionOutput {
            module: None,
            output: self.0.clone(),
        })
    }
}

#[derive(Clone)]
struct LiteralBuilder;

impl engine::CommandBuilder for LiteralBuilder {
    fn name(&self) -> String {
        "literal".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            <var> = {command} <literal>
                sets <var> to <literal>
        ", command=self.name()}
    }
    fn build(&self, _workdir: &Path, args: &Arguments) -> Result<engine::Command> {
        let value = args.expect_single_arg(self.name())?.to_owned();
        Ok(engine::Command::new_expression(LiteralExpression(
            value.into(),
        )))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(LiteralBuilder));
}
