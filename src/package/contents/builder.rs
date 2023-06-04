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

pub trait Builder {
    fn name(&self) -> String;
    fn help(&self) -> String;
    fn build(&self, state: &mut State, args: &[&str]) -> Result<Option<Box<dyn Module>>>;
}

fn builders() -> Vec<Box<dyn Builder>> {
    vec![
        // MANIFEST.
        Box::new(super::subdir::SubdirBuilder {}),
        Box::new(super::subdirs::SubdirsBuilder {}),
        Box::new(super::prefix::PrefixBuilder {}),
        Box::new(super::xdg_prefix::XdgCachePrefixBuilder {}),
        Box::new(super::xdg_prefix::XdgConfigPrefixBuilder {}),
        Box::new(super::xdg_prefix::XdgDataPrefixBuilder {}),
        Box::new(super::xdg_prefix::XdgStatePrefixBuilder {}),
        Box::new(super::tags::RequiresBuilder {}),
        Box::new(super::tags::ConflictsBuilder {}),
        // Files.
        Box::new(super::symlink::SymlinkBuilder {}),
        Box::new(super::symlink_tree::SymlinkTreeBuilder {}),
        Box::new(super::mkdir::MkDirBuilder {}),
        Box::new(super::copy::CopyBuilder {}),
        Box::new(super::output_file::OutputFileBuilder {}),
        Box::new(super::cat_glob::CatGlobIntoBuilder {}),
        Box::new(super::set_contents::SetContentsBuilder {}),
        Box::new(super::importer::ImporterBuilder {}),
        // Downloads.
        Box::new(super::fetch::FetchIntoBuilder {}),
        Box::new(super::fetch::FetchExeIntoBuilder {}),
        Box::new(super::git_clone::GitCloneBuilder {}),
        // Exec.
        Box::new(super::exec::PostInstallExecBuilder {}),
        Box::new(super::exec::PostInstallUpdateBuilder {}),
        // Control.
        Box::new(super::if_missing::IfMissingBuilder {}),
        // Deprecation.
        Box::new(super::deprecated::DeprecatedBuilder {}),
    ]
}

pub fn build(state: &mut State, args: &[&str]) -> Result<Option<Box<dyn Module>>> {
    if !state.enabled {
        return Ok(None);
    }
    let mut matched = Vec::<(String, Option<Box<dyn Module>>)>::new();
    for builder in builders() {
        match builder.build(state, args) {
            Ok(m) => matched.push((builder.name(), m)),
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
    for builder in builders() {
        help.push_str(&builder.help().trim_end());
        help.push('\n');
    }
    help
}
