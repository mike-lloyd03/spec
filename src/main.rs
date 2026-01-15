use anyhow::Result;
use clap::Parser;
use spec::{
    commands,
    types::{
        app::App,
        cli::{self, Cli},
    },
};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let app = App::new()?;

    match cli.command {
        cli::Command::Run(args) => commands::run(&app, &args),
        cli::Command::Unmanaged => commands::unmanaged(),
        cli::Command::Rollback(args) => commands::rollback(&app, &args),
        cli::Command::Verify(args) => commands::verify(&app, &args),
    }
}
