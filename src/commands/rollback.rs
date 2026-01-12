use color_eyre::eyre::{Result, bail};

use crate::{
    App,
    cli::{RollbackArgs, RunArgs},
    commands::run::process_services,
    db::types::Run,
};

pub fn rollback(app: &App, args: &RollbackArgs) -> Result<()> {
    if let Ok(last_run) = Run::get_previous(&app.db) {
        let run_args = RunArgs {
            dry_run: false,
            noconfirm: args.noconfirm,
        };

        process_services(app, &run_args, &last_run.data, &mut vec![])?;

        last_run.delete(&app.db)?;
    } else {
        bail!("No previous run to rollback to")
    }

    Ok(())
}
