mod adapters;
mod cli;
mod commands;
mod services;

use clap::Parser;
use color_eyre::Result;

use crate::cli::Cli;

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    match cli.command {
        cli::Command::Run { dry_run } => commands::run(dry_run),
        cli::Command::Unmanaged => commands::unmanaged(),
    }
}
