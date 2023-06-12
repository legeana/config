use anyhow::Result;
use indoc::formatdoc;

use crate::module::Module;

use super::builder;
use super::util;

pub struct SubdirBuilder;

impl builder::Builder for SubdirBuilder {
    fn name(&self) -> String {
        "subdir".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <subdirectory>
                load subdirectory configuration recursively
        ", command=self.name()}
    }
    fn build(&self, state: &mut builder::State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        let subdir = util::single_arg(&self.name(), args)?;
        let mut substate = builder::State {
            enabled: true,
            prefix: state.prefix.join(subdir),
        };
        let subroot = substate.prefix.src_dir.clone();
        let subconf = super::Configuration::new_sub(&mut substate, subroot)?;
        Ok(Some(Box::new(subconf)))
    }
}
