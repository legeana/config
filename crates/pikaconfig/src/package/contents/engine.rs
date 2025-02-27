use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow, bail};

use crate::module::ModuleBox;

use super::args::{Argument, Arguments};

pub(super) struct Context {
    pub enabled: bool,
    pub prefix: PathBuf,
    home_var: OsString,
    vars: HashMap<String, OsString>,
}

impl Context {
    pub(super) fn new() -> Self {
        let home = dirs::home_dir().expect("failed to determine home dir");
        let home_var: OsString = home.clone().into();
        Self {
            enabled: true,
            prefix: home,
            home_var,
            vars: HashMap::new(), // Variables are not inherited.
        }
    }
    pub(super) fn subdir(&self, path: impl AsRef<Path>) -> Self {
        Self {
            enabled: true,
            prefix: self.prefix.join(path.as_ref()),
            home_var: self.home_var.clone(),
            vars: HashMap::new(), // Variables are not inherited.
        }
    }
    pub(super) fn dst_path(&self, path: impl AsRef<Path>) -> PathBuf {
        self.prefix.join(path)
    }
    pub(super) fn expand_arg<'a>(&'a self, arg: &Argument) -> Result<OsString> {
        let get_var = |var: &str| -> Result<Option<&'a OsString>> {
            Ok(Some(
                self.get_var(var)
                    .ok_or_else(|| anyhow!("failed to resolve {var:?}"))?,
            ))
        };
        let home_dir = || -> Option<&Path> {
            // Must never return None, or ~ will not get expanded.
            // This leads to pretty nasty behaviour in the code.
            Some(self.home_var.as_ref())
        };
        let rendered = match arg {
            Argument::Raw(s) => return Ok(s.into()),
            Argument::OnlyVars(t) => shellexpand::path::env_with_context(t, get_var),
            Argument::VarsAndHome(t) => shellexpand::path::full_with_context(t, home_dir, get_var),
        };
        match rendered {
            Err(e) => Err(anyhow!("failed to expand {arg:?}: {e}")),
            Ok(Cow::Borrowed(p)) => Ok(p.as_os_str().to_owned()),
            Ok(Cow::Owned(p)) => Ok(p.into()),
        }
    }
    pub(super) fn expand_args(&self, args: impl AsRef<[Argument]>) -> Result<Vec<OsString>> {
        args.as_ref()
            .iter()
            .map(|arg| self.expand_arg(arg))
            .collect()
    }
    fn get_var(&self, var: &str) -> Option<&OsString> {
        match var {
            "HOME" => Some(&self.home_var),
            _ => self.vars.get(var),
        }
    }
    pub(super) fn set_var(
        &mut self,
        name: impl Into<String>,
        value: impl Into<OsString>,
    ) -> Result<()> {
        let name = name.into();
        let key = name.clone();
        let value = value.into();
        if self.vars.insert(key, value).is_some() {
            bail!("{name} is already set");
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(super) enum Command {
    Statement(StatementBox),
    Expression(ExpressionBox),
}

impl Command {
    pub(super) fn new_statement(statement: impl Statement + 'static) -> Self {
        Self::Statement(Box::new(statement))
    }
    pub(super) fn new_expression(expression: impl Expression + 'static) -> Self {
        Self::Expression(Box::new(expression))
    }
}

/// Builds a Statement.
/// This should be purely syntactical.
pub(super) trait CommandBuilder: Sync + Send {
    fn name(&self) -> String;
    fn help(&self) -> String;
    fn build(&self, workdir: &Path, args: &Arguments) -> Result<Command>;
}

pub(super) type CommandBuilderBox = Box<dyn CommandBuilder>;

pub(super) trait WithWrapperBuilder: Sync + Send {
    fn name(&self) -> String;
    fn help(&self) -> String;
    fn build(
        &self,
        workdir: &Path,
        args: &Arguments,
        statement: StatementBox,
    ) -> Result<StatementBox>;
}

pub(super) type WithWrapperBuilderBox = Box<dyn WithWrapperBuilder>;

pub(super) trait ConditionBuilder: Sync + Send {
    fn name(&self) -> String;
    fn help(&self) -> String;
    fn build(&self, workdir: &Path, args: &Arguments) -> Result<ConditionBox>;
}

pub(super) type ConditionBuilderBox = Box<dyn ConditionBuilder>;

pub(super) trait Condition: std::fmt::Debug {
    fn eval(&self, ctx: &Context) -> Result<bool>;
}

pub(super) type ConditionBox = Box<dyn Condition>;

/// Command creates a Module or modifies State.
pub(super) trait Statement: std::fmt::Debug {
    fn eval(&self, ctx: &mut Context) -> Result<Option<ModuleBox>>;
}

pub(super) type StatementBox = Box<dyn Statement>;

pub(super) struct ExpressionOutput {
    pub module: Option<ModuleBox>,
    pub output: OsString,
}

pub(super) trait Expression: std::fmt::Debug {
    fn eval(&self, ctx: &mut Context) -> Result<ExpressionOutput>;
}

pub(super) type ExpressionBox = Box<dyn Expression>;

pub(super) fn new_command(workdir: &Path, command: &str, args: &Arguments) -> Result<Command> {
    let builder = super::inventory::command(command)?;
    builder.build(workdir, args)
}

pub(super) fn new_with_wrapper(
    workdir: &Path,
    name: &str,
    args: &Arguments,
    statement: StatementBox,
) -> Result<StatementBox> {
    let builder = super::inventory::with_wrapper(name)?;
    builder.build(workdir, args, statement)
}

pub(super) fn new_condition(workdir: &Path, name: &str, args: &Arguments) -> Result<ConditionBox> {
    let builder = super::inventory::condition(name)?;
    builder.build(workdir, args)
}

pub fn help() -> String {
    let mut help = String::new();
    help.push_str("## Commands\n");
    for parser in super::inventory::commands() {
        help.push_str(parser.help().trim_end());
        help.push('\n');
    }
    help.push('\n');
    help.push_str("## Conditions\n");
    for condition in super::inventory::conditions() {
        help.push_str(condition.help().trim());
        help.push('\n');
    }
    help.push('\n');
    help.push_str("## with statement wrappers\n");
    for with_wrapper in super::inventory::with_wrappers() {
        help.push_str(with_wrapper.help().trim());
        help.push('\n');
    }
    help
}

#[derive(Debug)]
pub(super) struct VecStatement(pub Vec<StatementBox>);

impl Statement for VecStatement {
    fn eval(&self, ctx: &mut Context) -> Result<Option<ModuleBox>> {
        let mut modules: Vec<_> = Vec::new();
        for statement in &self.0 {
            if !ctx.enabled {
                break;
            }
            if let Some(module) = statement.eval(ctx)? {
                modules.push(module);
            }
        }
        if modules.is_empty() {
            return Ok(None);
        }
        Ok(Some(Box::new(modules)))
    }
}

#[cfg(test)]
mod tests {
    use super::super::args::args;
    use super::*;

    macro_rules! os {
        ($s:literal) => {
            std::ffi::OsStr::new($s)
        };
    }

    #[test]
    fn test_context_expand_arg_raw() {
        let ctx = Context::new();
        assert_eq!(
            ctx.expand_arg(&Argument::Raw("hello".into())).unwrap(),
            os!("hello")
        );
        assert_eq!(
            ctx.expand_arg(&Argument::Raw("~/hello".into())).unwrap(),
            os!("~/hello")
        );
        assert_eq!(
            ctx.expand_arg(&Argument::Raw("${hello}".into())).unwrap(),
            os!("${hello}")
        );
    }

    #[test]
    fn test_context_expand_arg_only_vars() {
        let mut ctx = Context::new();
        ctx.set_var("hello", "world").unwrap();

        assert_eq!(
            ctx.expand_arg(&Argument::OnlyVars("hello".into())).unwrap(),
            os!("hello")
        );
        assert_eq!(
            ctx.expand_arg(&Argument::OnlyVars("${hello}".into()))
                .unwrap(),
            os!("world")
        );
    }

    #[test]
    fn test_context_expand_arg_unset_var() {
        let ctx = Context::new();
        assert!(
            ctx.expand_arg(&Argument::OnlyVars("$hello".into()))
                .err()
                .unwrap()
                .to_string()
                .contains("hello")
        );
    }

    #[test]
    fn test_context_expand_arg_vars_and_home() {
        let mut ctx = Context::new();
        ctx.set_var("hello", "world").unwrap();
        assert_eq!(
            ctx.expand_arg(&Argument::VarsAndHome("hello".into()))
                .unwrap(),
            os!("hello")
        );
        assert_eq!(
            ctx.expand_arg(&Argument::VarsAndHome("${hello}".into()))
                .unwrap(),
            os!("world")
        );

        let mut want = dirs::home_dir().unwrap().as_os_str().to_owned();
        want.push("/subdir");
        assert_eq!(
            ctx.expand_arg(&Argument::VarsAndHome("~/subdir".into()))
                .unwrap(),
            want
        );
    }

    #[test]
    fn test_context_expand_args() {
        let mut ctx = Context::new();
        ctx.set_var("var_1", "val_1").unwrap();
        ctx.set_var("var_2", "val_2").unwrap();

        assert_eq!(
            ctx.expand_args(args!["hello", @"$var_1", ~"$var_2"])
                .unwrap(),
            vec![os!("hello"), os!("val_1"), os!("val_2")]
        );
    }
}
