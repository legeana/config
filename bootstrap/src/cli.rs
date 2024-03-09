use std::env;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

const CONFIG_ROOT_ENV: &str = "PIKACONFIG_CONFIG_ROOT";

#[derive(Debug, Parser)]
pub struct Cli {
    #[clap(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
    #[clap(short = 'd', long)]
    pub no_update: bool,
    #[clap(subcommand)]
    pub command: Commands,
    #[clap(
        short = 'k',
        long,
        help = "Don't interrupt installation process if a package fails"
    )]
    pub keep_going: bool,
    #[clap(long, help = "Don't install user dependencies")]
    pub no_user_deps: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Install {},
    Update {},
    SystemInstall {},
    Uninstall {},
    ManifestHelp {},
    Tags {},
    List {},
}

pub fn config_root() -> Result<PathBuf> {
    let config_root = env::var_os(CONFIG_ROOT_ENV)
        .with_context(|| format!("failed to read {CONFIG_ROOT_ENV}, use setup"))?;
    Ok(config_root.into())
}

pub fn parse() -> Cli {
    Cli::parse()
}
