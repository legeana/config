use std::path::Path;

use crate::module::ModuleBox;
use crate::tera_helper;

use super::ast;
use super::engine;
use super::inventory;
use super::util;

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
    cmd: ast::StatementBox,
}

impl IfOsStatement {
    fn is_os(&self) -> bool {
        is_os(self.os)
    }
}

impl ast::Statement for IfOsStatement {
    fn eval(&self, state: &mut engine::State) -> Result<Option<ModuleBox>> {
        if !self.is_os() {
            return Ok(None);
        }
        self.cmd.eval(state)
    }
}

#[derive(Clone)]
struct IfOsParser {
    os: &'static str,
}

impl IfOsParser {
    fn command(&self) -> String {
        format!("if_{}", self.os)
    }
}

impl ast::Parser for IfOsParser {
    fn name(&self) -> String {
        self.command()
    }
    fn help(&self) -> String {
        formatdoc! {"
            {command} <command> [<args>...]
                execute a MANIFEST <command> only if os (or family) is {os}
        ", os=self.os, command=self.command()}
    }
    fn parse(&self, workdir: &Path, args: &[&str]) -> Result<ast::StatementBox> {
        let (empty, cmd_args) = util::multiple_args(&self.command(), args, 0)?;
        assert!(empty.is_empty());
        Ok(Box::new(IfOsStatement {
            os: self.os,
            cmd: ast::parse(workdir, cmd_args)?,
        }))
    }
}

impl inventory::RenderHelper for IfOsParser {
    fn register_render_helper(&self, tera: &mut tera::Tera) {
        let name = format!("is_{}", self.os);
        let os = self.os.to_owned();
        tera.register_function(&name, tera_helper::wrap_nil(move || Ok(is_os(&os))));
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    let parsers = [
        IfOsParser { os: "macos" },
        IfOsParser { os: "linux" },
        IfOsParser { os: "unix" },
        IfOsParser { os: "windows" },
    ];
    for parser in parsers {
        registry.register_parser(Box::new(parser.clone()));
        registry.register_render_helper(Box::new(parser));
    }
}
