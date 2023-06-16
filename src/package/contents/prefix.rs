use anyhow::Result;
use indoc::formatdoc;

use crate::module::Module;

use super::builder;
use super::util;

#[derive(Clone)]
struct PrefixBuilder {
    prefix: String,
}

impl builder::Builder for PrefixBuilder {
    fn build(&self, state: &mut builder::State) -> Result<Option<Box<dyn Module>>> {
        state
            .prefix
            .set(shellexpand::tilde(&self.prefix).as_ref().into());
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
    fn parse(&self, args: &[&str]) -> Result<Box<dyn builder::Builder>> {
        let prefix = util::single_arg(&self.name(), args)?.to_owned();
        Ok(Box::new(PrefixBuilder { prefix }))
    }
}

pub fn commands() -> Vec<Box<dyn builder::Parser>> {
    vec![Box::new(PrefixParser {})]
}
