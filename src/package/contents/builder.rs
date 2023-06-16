use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use thiserror::Error;

use crate::module::Module;

#[derive(Error, Debug)]
pub enum Error {
    #[error("builder {builder}: unsupported command {command}")]
    UnsupportedCommand { builder: String, command: String },
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

/// Parser transforms a statement into a Builder.
/// This should be purely syntactical.
pub trait Parser {
    fn name(&self) -> String;
    fn help(&self) -> String;
    fn parse(&self, workdir: &Path, args: &[&str]) -> Result<Box<dyn Builder>>;
}

/// Builder is creates a Module or modifies State.
pub trait Builder: std::fmt::Debug {
    fn build(&self, state: &mut State) -> Result<Option<Box<dyn Module>>>;
}

fn parsers() -> Vec<Box<dyn Parser>> {
    let result: Vec<Vec<Box<dyn Parser>>> = vec![
        // MANIFEST.
        super::subdir::commands(),
        super::prefix::commands(),
        super::xdg_prefix::commands(),
        super::tags::commands(),
        // Files.
        super::symlink::commands(),
        super::symlink_tree::commands(),
        super::mkdir::commands(),
        super::copy::commands(),
        super::output_file::commands(),
        super::cat_glob::commands(),
        super::set_contents::commands(),
        super::importer::commands(),
        // Downloads.
        super::fetch::commands(),
        super::git_clone::commands(),
        // Exec.
        super::exec::commands(),
        // Control.
        super::if_missing::commands(),
        super::if_os::commands(),
        // Deprecation.
        super::deprecated::commands(),
    ];
    result.into_iter().flatten().collect()
}

pub fn parse(workdir: &Path, args: &[&str]) -> Result<Box<dyn Builder>> {
    let mut matched: Vec<(String, Box<dyn Builder>)> = Vec::new();
    for parser in parsers() {
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

pub fn help() -> String {
    let mut help = String::new();
    for parser in parsers() {
        help.push_str(parser.help().trim_end());
        help.push('\n');
    }
    help
}
