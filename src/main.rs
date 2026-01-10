use clap::Parser;
use color_eyre::Result;
use spec::{App, cli, commands};

use spec::cli::Cli;

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();
    let app = App::new()?;

    match cli.command {
        cli::Command::Run(args) => commands::run(&app, &args),
        cli::Command::Unmanaged => commands::unmanaged(),
        cli::Command::Rollback(args) => commands::rollback(&app, &args),
        cli::Command::Verify(args) => commands::verify(&app, &args),
    }
}
