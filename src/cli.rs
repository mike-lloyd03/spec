use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    /// The command to run
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Read the service configuration and implement it on the system
    Run {
        /// Shows what would be done without doing anything
        #[arg(short, long)]
        dry_run: bool,
    },

    /// List unmanaged systemd services
    Unmanaged,
}
