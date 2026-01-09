mod adapters;
mod cli;
mod commands;
mod config;
mod db;
mod services;

use clap::Parser;
use color_eyre::Result;

use crate::{cli::Cli, config::Config};

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    let config = Config::load(cli.config_dir)?;

    match cli.command {
        cli::Command::Run(args) => commands::run(&args, config),
        cli::Command::Unmanaged => commands::unmanaged(),
        cli::Command::Rollback(args) => commands::rollback(&args),
    }
}
