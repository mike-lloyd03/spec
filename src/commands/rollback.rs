use anyhow::{Result, bail};

use crate::{
    commands::run::process_services,
    types::app::App,
    types::cli::{RollbackArgs, RunArgs},
    types::run::Run,
};

pub fn rollback(app: &App, args: &RollbackArgs) -> Result<()> {
    if let Ok(last_run) = Run::get_previous(&app.db) {
        let run_args = RunArgs {
            dry_run: false,
            noconfirm: args.noconfirm,
            force: false,
        };

        let mut _success = true;

        process_services(app, &run_args, &last_run.data, &mut vec![], &mut _success)?;

        last_run.delete(&app.db)?;
    } else {
        bail!("No previous run to rollback to")
    }

    Ok(())
}
