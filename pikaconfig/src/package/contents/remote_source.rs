use std::path::Path;

use anyhow::{Context, Result};
use indoc::formatdoc;

use super::args::Arguments;
use super::engine;
use super::inventory;

#[derive(Debug)]
struct RemoteArchiveExpression {
    filename: String,
    url: String,
}

impl engine::Expression for RemoteArchiveExpression {
    fn eval(&self, _ctx: &mut engine::Context) -> Result<engine::ExpressionOutput> {
        todo!("{} {}", self.filename, self.url)
    }
}

#[derive(Clone)]
struct RemoteArchiveBuilder;

impl engine::CommandBuilder for RemoteArchiveBuilder {
    fn name(&self) -> String {
        "remote_archive".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            <directory> = {command} <filename> <url>
                Downloads and unpacks remote archive,
                returns a path to unpacked directory.
        ", command=self.name()}
    }
    fn build(&self, _workdir: &Path, args: &Arguments) -> Result<engine::Command> {
        let (filename, url) = args.expect_double_arg(self.name())?;
        Ok(engine::Command::new_expression(RemoteArchiveExpression {
            filename: filename.expect_raw().context("filename")?.to_owned(),
            url: url.expect_raw().context("url")?.to_owned(),
        }))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_command(Box::new(RemoteArchiveBuilder));
}
