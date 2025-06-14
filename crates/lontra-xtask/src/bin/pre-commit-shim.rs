use anyhow::Result;
use clap::Parser;
use lontra_xtask::install::install_self_as_git_hook;
use xshell::Shell;
use xshell::cmd;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(short, long)]
    quiet: bool,
    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    #[clap(long)]
    install: bool,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    cli::logconfig::init(args.quiet, args.verbose)?;
    let sh = Shell::new()?;

    if args.install {
        return install_self_as_git_hook(&sh, "pre-commit");
    }

    cmd!(sh, "cargo xtask pre-commit").run()?;

    Ok(())
}
