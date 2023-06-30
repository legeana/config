use std::path::Path;

use anyhow::Result;
use indoc::formatdoc;

use crate::module::Module;

use super::builder;
use super::inventory;
use super::util;

#[derive(Debug)]
struct PrefixStatement {
    prefix: String,
}

impl builder::Statement for PrefixStatement {
    fn eval(&self, state: &mut builder::State) -> Result<Option<Box<dyn Module>>> {
        state.prefix = shellexpand::tilde(&self.prefix).as_ref().into();
        Ok(None)
    }
}

#[derive(Clone)]
struct PrefixParser;

impl builder::Parser for PrefixParser {
    fn name(&self) -> String {
        "prefix".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <directory>
                set current installation prefix to <directory>
        ", command=self.name()}
    }
    fn parse(&self, _workdir: &Path, args: &[&str]) -> Result<Box<dyn builder::Statement>> {
        let prefix = util::single_arg(&self.name(), args)?.to_owned();
        Ok(Box::new(PrefixStatement { prefix }))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_parser(Box::new(PrefixParser {}));
}
