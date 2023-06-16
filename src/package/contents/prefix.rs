use anyhow::Result;
use indoc::formatdoc;

use crate::module::Module;

use super::builder;
use super::util;

#[derive(Clone)]
struct PrefixBuilder;

impl builder::Builder for PrefixBuilder {
    fn name(&self) -> String {
        "prefix".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <directory>
                set current installation prefix to <directory>
        ", command=self.name()}
    }
    fn build(&self, state: &mut builder::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        let prefix = util::single_arg(&self.name(), args)?;
        state.prefix.set(shellexpand::tilde(prefix).as_ref().into());
        Ok(None)
    }
}

pub fn commands() -> Vec<Box<dyn builder::Parser>> {
    vec![Box::new(PrefixBuilder {})]
}
