use std::path::Path;

use crate::module::ModuleBox;
use crate::tera_helper;

use super::args::Arguments;
use super::engine::{self, ConditionBuilder};
use super::inventory;

use anyhow::Result;
use indoc::formatdoc;

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

#[derive(Clone)]
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
    fn register_render_helper(&self, tera: &mut tera::Tera) {
        let name = self.name();
        let is_os = IsOs(self.0);
        tera.register_function(&name, tera_helper::wrap_nil(move || Ok(is_os.check())));
    }
}

#[derive(Debug)]
struct IfOsStatement {
    is_os: IsOs,
    cmd: engine::StatementBox,
}

impl engine::Statement for IfOsStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        if !self.is_os.check() {
            return Ok(None);
        }
        self.cmd.eval(ctx)
    }
}

#[derive(Clone)]
struct IfOsBuilder {
    os: &'static str,
}

impl IfOsBuilder {
    fn command(&self) -> String {
        format!("if_{}", self.os)
    }
}

impl engine::CommandBuilder for IfOsBuilder {
    fn name(&self) -> String {
        self.command()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <command> [<args>...]
                execute a MANIFEST <command> only if os (or family) is {os}
        ", os=self.os, command=self.command()}
    }
    fn build(&self, workdir: &Path, args: &Arguments) -> Result<engine::StatementBox> {
        let (empty, cmd_args) = args.expect_variadic_args(self.command(), 0)?;
        assert!(empty.is_empty());
        let cmd_args: Vec<_> = cmd_args.iter().map(String::as_str).collect();
        Ok(Box::new(IfOsStatement {
            is_os: IsOs(self.os),
            cmd: engine::parse_args(workdir, &cmd_args)?,
        }))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    let oss = ["macos", "linux", "unix", "windows"];
    for os in oss {
        registry.register_command(Box::new(IfOsBuilder { os }));
        registry.register_condition(Box::new(IsOsBuilder(os)));
        registry.register_render_helper(Box::new(IsOsBuilder(os)));
    }
}
