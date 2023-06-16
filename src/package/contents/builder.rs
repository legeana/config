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

pub trait BoxParserClone {
    fn parser_clone(&self) -> Box<dyn Parser>;
}

impl<T> BoxParserClone for T
where
    T: Parser + Clone + 'static,
{
    fn parser_clone(&self) -> Box<dyn Parser> {
        Box::new(self.clone())
    }
}

/// Parser transforms a statement into a Builder.
/// This should be purely syntactical.
pub trait Parser: BoxParserClone {
    fn name(&self) -> String;
    fn help(&self) -> String;
    fn parse(&self, args: &[&str]) -> Result<Box<dyn Builder>>;
    // Compatibility functions.
    fn build(&self, state: &mut State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
        let builder = self.parse(args)?;
        builder.build(state)
    }
}

/// Builder is creates a Module or modifies State.
pub trait Builder {
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

struct DelegateBuilder {
    args: Vec<String>,
}

impl Builder for DelegateBuilder {
    fn build(&self, state: &mut State) -> Result<Option<Box<dyn Module>>> {
        let args: Vec<_> = self.args.iter().map(String::as_ref).collect();
        build(state, &args)
    }
}

pub fn parse(args: &[&str]) -> Result<Box<dyn Builder>> {
    Ok(Box::new(DelegateBuilder {
        args: args.iter().map(|&s| s.to_owned()).collect(),
    }))
}

pub fn build(state: &mut State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
    if !state.enabled {
        return Ok(None);
    }
    let mut matched = Vec::<(String, Option<Box<dyn Module>>)>::new();
    for parser in parsers() {
        match parser.build(state, args) {
            Ok(m) => matched.push((parser.name(), m)),
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
            "{:?} matched multiple builders: {:?}",
            args,
            matched
                .iter()
                .map(|(builder, _)| builder)
                .collect::<Vec<_>>(),
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
