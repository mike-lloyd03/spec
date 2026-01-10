use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    /// The command to run
    #[command(subcommand)]
    pub command: Command,

    /// Set an alternate config directory
    #[arg(short, long)]
    pub config_dir: Option<String>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Read the service configuration and implement it on the system
    Run(RunArgs),

    /// List unmanaged systemd services
    Unmanaged,

    /// Rollback to a previous state
    Rollback(RollbackArgs),
}

#[derive(Args)]
pub struct RunArgs {
    /// Shows what would be done without doing anything
    #[arg(short, long)]
    pub dry_run: bool,

    /// Overwrite existing files without confirmation
    #[arg(long)]
    pub noconfirm: bool,
}

#[derive(Args)]
pub struct RollbackArgs {
    /// Overwrite existing files without confirmation
    #[arg(long)]
    pub noconfirm: bool,
}
