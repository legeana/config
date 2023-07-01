use std::path::Path;

use anyhow::Result;

use crate::module::ModuleBox;

use super::ast;
use super::inventory;
use super::util::check_command;

#[derive(Debug)]
struct NoOpStatement;

impl ast::Statement for NoOpStatement {
    fn eval(&self, _state: &mut ast::State) -> Result<Option<ModuleBox>> {
        Ok(None)
    }
}

#[derive(Clone)]
struct DeprecatedParser;

impl ast::Parser for DeprecatedParser {
    fn name(&self) -> String {
        "deprecated commands, do not use".to_owned()
    }
    fn help(&self) -> String {
        "DEPRECATED: N/A".to_owned()
    }
    fn parse(&self, _workdir: &Path, args: &[&str]) -> Result<ast::StatementBox> {
        /*if check_command("<deprecated>", args).is_ok() {
            log::warn!(
                "{:?}: <deprecated> is unsupported",
                configuration.root
            );
            return Ok(());
        }*/
        check_command(&self.name(), args).map(|_| ())?;
        Ok(Box::new(NoOpStatement {}))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_parser(Box::new(DeprecatedParser {}));
}
