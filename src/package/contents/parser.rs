use std::path::{Path, PathBuf};

use crate::package::contents::Configuration;

use anyhow::anyhow;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("parser {parser}: unsupported command {command}")]
    UnsupportedCommand { parser: String, command: String },
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Prefix {
    pub base: PathBuf,
    pub current: PathBuf,
}

impl Prefix {
    fn new() -> Self {
        let home = dirs::home_dir().expect("failed to determine home dir");
        Self {
            base: home.clone(),
            current: home,
        }
    }
    pub fn set(&mut self, current: PathBuf) {
        self.current = current;
    }
    pub fn join<P: AsRef<Path>>(&self, subdir: P) -> Self {
        let sub = self.current.join(subdir);
        Self {
            base: sub.clone(),
            current: sub,
        }
    }
}

pub struct State {
    pub prefix: Prefix,
}

impl State {
    pub fn new() -> Self {
        Self {
            prefix: Prefix::new(),
        }
    }
}

pub trait Parser {
    fn name(&self) -> &'static str;
    fn help(&self) -> &'static str;
    fn parse(
        &self,
        state: &mut State,
        configuration: &mut Configuration,
        args: &[&str],
    ) -> Result<()>;
}

fn parsers() -> Vec<Box<dyn Parser>> {
    vec![
        Box::new(super::subdir::SubdirParser {}),
        Box::new(super::subdirs::SubdirsParser {}),
        Box::new(super::prefix::PrefixParser {}),
        Box::new(super::xdg_prefix::XdgCachePrefixParser {}),
        Box::new(super::xdg_prefix::XdgConfigPrefixParser {}),
        Box::new(super::xdg_prefix::XdgDataPrefixParser {}),
        Box::new(super::xdg_prefix::XdgStatePrefixParser {}),
        Box::new(super::tags::RequiresParser {}),
        Box::new(super::tags::ConflictsParser {}),
        Box::new(super::symlink::SymlinkParser {}),
        Box::new(super::symlink_tree::SymlinkTreeParser {}),
        Box::new(super::mkdir::MkDirParser {}),
        Box::new(super::copy::CopyParser {}),
        Box::new(super::output_file::OutputFileParser {}),
        Box::new(super::cat_glob::CatGlobIntoParser {}),
        Box::new(super::importer::ImporterParser {}),
        Box::new(super::exec::PostInstallExecParser {}),
        Box::new(super::deprecated::DeprecatedParser {}),
    ]
}

pub fn parse(
    state: &mut State,
    configuration: &mut Configuration,
    args: &[&str],
) -> anyhow::Result<()> {
    let mut matched = Vec::<String>::new();
    for parser in parsers() {
        match parser.parse(state, configuration, args) {
            Ok(()) => {
                // Success.
                matched.push(parser.name().to_string());
                continue;
            }
            Err(Error::UnsupportedCommand {
                parser: _,
                command: _,
            }) => {
                // Try another parser.
                continue;
            }
            Err(Error::Other(error)) => {
                return Err(error);
            }
        }
    }
    match matched.len() {
        0 => Err(anyhow!("unsupported command {:?}", args)),
        1 => Ok(()),
        _ => Err(anyhow!(
            "{:?} matched multiple parsers: {:?}",
            args,
            matched,
        )),
    }
}

pub fn help() -> String {
    let mut help = String::new();
    for parser in parsers() {
        help.push_str(parser.help());
        help.push('\n');
    }
    help
}
