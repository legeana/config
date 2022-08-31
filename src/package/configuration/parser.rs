use std::path::{Path, PathBuf};

use crate::package::configuration::Configuration;

use anyhow::anyhow;
use dirs;
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
        return Self {
            base: home.clone(),
            current: home,
        };
    }
    pub fn set(&mut self, current: PathBuf) {
        self.current = current;
    }
    pub fn join<P: AsRef<Path>>(&self, subdir: P) -> Self {
        let sub = self.current.join(subdir);
        return Self {
            base: sub.clone(),
            current: sub,
        };
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
        // TODO: subdirs
        Box::new(super::prefix::PrefixParser {}),
        // TODO: xdg_cache_prefix
        // TODO: xdg_config_prefix
        // TODO: xdg_data_prefix
        // TODO: xdg_state_prefix
        // TODO: requires
        // TODO: conflicts
        // TODO: install_system_package
        // TODO: install_pacman_package
        // TODO: install_apt_package
        // TODO: install_brew_package
        // TODO: install_pip_user_package
        // TODO: symlink
        // TODO: symlink_tree
        // TODO: copy
        // TODO: output_file
        // TODO: cat_glob
        // TODO: import_from
        // TODO: post_install_exec
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
        0 => {
            println!("unsupported command {:?}", args);
            Ok(())
            // TODO: Err(anyhow!("unsupported command {:?}", args));
        }
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
    return help;
}
