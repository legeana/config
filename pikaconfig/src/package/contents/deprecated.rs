use std::path::Path;

use anyhow::Result;

use crate::module::ModuleBox;

use super::engine;
use super::inventory;

#[derive(Debug)]
struct NoOpStatement;

impl engine::Statement for NoOpStatement {
    fn eval(&self, _ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        Ok(None)
    }
}

#[derive(Clone)]
struct DeprecatedBuilder(&'static str);

impl engine::CommandBuilder for DeprecatedBuilder {
    fn name(&self) -> String {
        "deprecated commands, do not use".to_owned()
    }
    fn help(&self) -> String {
        "DEPRECATED: N/A".to_owned()
    }
    fn parse(&self, workdir: &Path, _args: &[&str]) -> Result<engine::StatementBox> {
        log::warn!(
            "{workdir:?}: {:?} is unsupported",
            self.0
        );
        Ok(Box::new(NoOpStatement {}))
    }
}

pub fn register(_registry: &mut dyn inventory::Registry) {
    //registry.register_command(Box::new(DeprecatedBuilder("<deprecated>")));
}
