use std::path::Path;

use super::args::Arguments;
use super::engine;
use super::engine::ConditionBuilder as _;
use super::inventory;

use anyhow::Result;
use indoc::formatdoc;
use minijinja::Environment;

#[derive(Debug)]
struct IsOs(&'static str);

impl IsOs {
    fn check(&self) -> bool {
        // We don't have to check this in runtime as this never changes.
        // In fact, it's even beneficial to check this during build to support *Prefix.
        self.0 == std::env::consts::FAMILY || self.0 == std::env::consts::OS
    }
}

impl engine::Condition for IsOs {
    fn eval(&self, _ctx: &engine::Context) -> Result<bool> {
        Ok(self.check())
    }
}

#[derive(Copy, Clone)]
struct IsOsBuilder(&'static str);

impl engine::ConditionBuilder for IsOsBuilder {
    fn name(&self) -> String {
        format!("is_{}", self.0)
    }
    fn help(&self) -> String {
        formatdoc! {"
            {condition}
                true if os is {os}
        ", condition=self.name(), os=self.0}
    }
    fn build(&self, _workdir: &Path, args: &Arguments) -> Result<engine::ConditionBox> {
        args.expect_no_args(self.name())?;
        Ok(Box::new(IsOs(self.0)))
    }
}

impl inventory::RenderHelper for IsOsBuilder {
    fn register_render_helper2(&self, env: &mut Environment) {
        let is_os = IsOs(self.0);
        env.add_function(self.name(), move || is_os.check());
    }
}

pub(super) fn register(registry: &mut dyn inventory::Registry) {
    let builders = [
        IsOsBuilder("macos"),
        IsOsBuilder("linux"),
        IsOsBuilder("unix"),
        IsOsBuilder("windows"),
    ];
    for builder in builders {
        registry.register_condition(Box::new(builder));
        registry.register_render_helper(Box::new(builder));
    }
}
