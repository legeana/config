use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use thiserror::Error;

use crate::module::Module;

#[derive(Error, Debug)]
pub enum Error {
    #[error("parser {parser}: unsupported command {command}")]
    UnsupportedCommand { parser: String, command: String },
}

struct PrefixNewGuard;

pub struct Prefix {
    _private_constructor_helper: PrefixNewGuard,
    pub src_dir: PathBuf,
    pub dst_dir: PathBuf,
}

impl Prefix {
    fn new(src_dir: PathBuf) -> Self {
        Self {
            _private_constructor_helper: PrefixNewGuard,
            src_dir,
            dst_dir: dirs::home_dir().expect("failed to determine home dir"),
        }
    }
    pub fn set(&mut self, dst_dir: PathBuf) {
        self.dst_dir = dst_dir;
    }
    pub fn join<P: AsRef<Path>>(&self, subdir: P) -> Self {
        Self {
            _private_constructor_helper: PrefixNewGuard,
            src_dir: self.src_path(subdir.as_ref()),
            dst_dir: self.dst_path(subdir.as_ref()),
        }
    }
    pub fn src_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.src_dir.join(path)
    }
    pub fn dst_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.dst_dir.join(path)
    }
}

pub struct State {
    pub enabled: bool,
    pub prefix: Prefix,
}

impl State {
    pub fn new(src: PathBuf) -> Self {
        Self {
            enabled: true,
            prefix: Prefix::new(src),
        }
    }
}

pub trait Parser {
    fn name(&self) -> &'static str;
    fn help(&self) -> &'static str;
    fn parse(
        &self,
        state: &mut State,
        configuration: &super::Configuration,
        args: &[&str],
    ) -> Result<Option<Box<dyn Module>>>;
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
        Box::new(super::set_contents::SetContentsParser {}),
        Box::new(super::importer::ImporterParser {}),
        Box::new(super::fetch::FetchIntoParser {}),
        Box::new(super::git_clone::GitCloneParser {}),
        Box::new(super::exec::PostInstallExecParser {}),
        Box::new(super::exec::PostInstallUpdateParser {}),
        Box::new(super::deprecated::DeprecatedParser {}),
    ]
}

pub fn parse(
    state: &mut State,
    configuration: &super::Configuration,
    args: &[&str],
) -> Result<Option<Box<dyn Module>>> {
    if !state.enabled {
        return Ok(None);
    }
    let mut matched = Vec::<(String, Option<Box<dyn Module>>)>::new();
    for parser in parsers() {
        match parser.parse(state, configuration, args) {
            Ok(m) => matched.push((parser.name().to_string(), m)),
            Err(err) => {
                match err.downcast_ref::<Error>() {
                    Some(Error::UnsupportedCommand {
                        parser: _,
                        command: _,
                    }) => {
                        // Try another parser.
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

pub fn help() -> String {
    let mut help = String::new();
    for parser in parsers() {
        help.push_str(parser.help());
        help.push('\n');
    }
    help
}
