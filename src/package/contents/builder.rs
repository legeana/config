use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use tera::Tera;
use thiserror::Error;

use crate::module::Module;

#[derive(Error, Debug)]
pub enum Error {
    #[error("builder {builder}: unsupported command {command}")]
    UnsupportedCommand { builder: String, command: String },
}

pub struct State {
    pub enabled: bool,
    pub prefix: PathBuf,
}

impl State {
    pub fn new() -> Self {
        Self {
            enabled: true,
            prefix: dirs::home_dir().expect("failed to determine home dir"),
        }
    }
    pub fn dst_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.prefix.join(path)
    }
}

pub trait RenderHelper: Sync + Send {
    /// [Optional] Register Tera helper.
    fn register_render_helper(&self, tera: &mut Tera) -> Result<()> {
        let _ = tera;
        Ok(())
    }
}

/// Parses a Statement.
/// This should be purely syntactical.
pub trait Parser: Sync + Send {
    fn name(&self) -> String;
    fn help(&self) -> String;
    fn parse(&self, workdir: &Path, args: &[&str]) -> Result<Box<dyn Statement>>;
}

/// Command creates a Module or modifies State.
pub trait Statement: std::fmt::Debug {
    fn eval(&self, state: &mut State) -> Result<Option<Box<dyn Module>>>;
}

pub fn parse(workdir: &Path, args: &[&str]) -> Result<Box<dyn Statement>> {
    let mut matched: Vec<(String, Box<dyn Statement>)> = Vec::new();
    for parser in super::inventory::parsers() {
        match parser.parse(workdir, args) {
            Ok(builder) => matched.push((parser.name(), builder)),
            Err(err) => {
                match err.downcast_ref::<Error>() {
                    Some(Error::UnsupportedCommand {
                        builder: _,
                        command: _,
                    }) => {
                        // Try another builder.
                        continue;
                    }
                    _ => {
                        return Err(err);
                    }
                }
            }
        }
    }
    match matched.len() {
        0 => Err(anyhow!("unsupported command {:?}", args)),
        1 => Ok(matched.pop().unwrap().1),
        _ => Err(anyhow!(
            "{:?} matched multiple parsers: {:?}",
            args,
            matched.iter().map(|(parser, _)| parser).collect::<Vec<_>>(),
        )),
    }
}

pub fn register_render_helpers(tera: &mut Tera) -> Result<()> {
    for rh in super::inventory::render_helpers() {
        rh.register_render_helper(tera)?;
    }
    Ok(())
}

pub fn help() -> String {
    let mut help = String::new();
    for parser in super::inventory::parsers() {
        help.push_str(parser.help().trim_end());
        help.push('\n');
    }
    help
}
