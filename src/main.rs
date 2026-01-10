mod adapters;
mod cli;
mod commands;
mod db;
mod services;

use clap::Parser;
use color_eyre::Result;
use spec::App;

use crate::cli::Cli;

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();
    let app = App::new()?;

    match cli.command {
        cli::Command::Run(args) => commands::run(&app, &args),
        cli::Command::Unmanaged => commands::unmanaged(),
        cli::Command::Rollback(args) => commands::rollback(&app, &args),
    }
}
