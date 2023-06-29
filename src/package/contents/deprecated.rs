use std::path::Path;

use anyhow::Result;

use crate::module::Module;

use super::builder;
use super::util::check_command;

#[derive(Debug)]
struct NoOpStatement;

impl builder::Statement for NoOpStatement {
    fn eval(&self, _state: &mut builder::State) -> Result<Option<Box<dyn Module>>> {
        Ok(None)
    }
}

#[derive(Clone)]
struct DeprecatedParser;

impl builder::Parser for DeprecatedParser {
    fn name(&self) -> String {
        "deprecated commands, do not use".to_owned()
    }
    fn help(&self) -> String {
        "DEPRECATED: N/A".to_owned()
    }
    fn parse(&self, _workdir: &Path, args: &[&str]) -> Result<Box<dyn builder::Statement>> {
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

pub fn commands() -> Vec<Box<dyn builder::Parser>> {
    vec![Box::new(DeprecatedParser {})]
}
