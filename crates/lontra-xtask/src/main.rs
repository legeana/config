use anyhow::Result;
use clap::{Parser, Subcommand};
use lontra_xtask as xt;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(short, long)]
    quiet: bool,
    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    InstallGitHooks,
    PreCommit,
    SqlxPrepare,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    cli::logconfig::init(args.quiet, args.verbose)?;
    match args.command {
        Commands::InstallGitHooks => {
            xt::pre_commit::install()?;
            Ok(())
        }
        Commands::PreCommit => xt::pre_commit::run(),
        Commands::SqlxPrepare => xt::sqlx::prepare(),
    }
}
