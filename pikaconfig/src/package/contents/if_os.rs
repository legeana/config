use std::path::Path;

use crate::module::ModuleBox;
use crate::tera_helper;

use super::args::Arguments;
use super::engine;
use super::inventory;

use anyhow::Result;
use indoc::formatdoc;

fn is_os(os: &str) -> bool {
    // We don't have to check this in runtime as this never changes.
    // In fact, it's even beneficial to check this during build to support *Prefix.
    os == std::env::consts::FAMILY || os == std::env::consts::OS
}

#[derive(Debug)]
struct IfOsStatement {
    os: &'static str,
    cmd: engine::StatementBox,
}

impl IfOsStatement {
    fn is_os(&self) -> bool {
        is_os(self.os)
    }
}

impl engine::Statement for IfOsStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        if !self.is_os() {
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
        let (empty, cmd_args) = args.expect_variadic_args(&self.command(), 0)?;
        assert!(empty.is_empty());
        let cmd_args: Vec<_> = cmd_args.iter().map(String::as_str).collect();
        Ok(Box::new(IfOsStatement {
            os: self.os,
            cmd: engine::parse_args(workdir, &cmd_args)?,
        }))
    }
}

impl inventory::RenderHelper for IfOsBuilder {
    fn register_render_helper(&self, tera: &mut tera::Tera) {
        let name = format!("is_{}", self.os);
        let os = self.os.to_owned();
        tera.register_function(&name, tera_helper::wrap_nil(move || Ok(is_os(&os))));
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    let parsers = [
        IfOsBuilder { os: "macos" },
        IfOsBuilder { os: "linux" },
        IfOsBuilder { os: "unix" },
        IfOsBuilder { os: "windows" },
    ];
    for parser in parsers {
        registry.register_command(Box::new(parser.clone()));
        registry.register_render_helper(Box::new(parser));
    }
}
