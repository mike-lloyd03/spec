use color_eyre::eyre::{Result, bail};
use spec::App;

use crate::{
    cli::{RollbackArgs, RunArgs},
    commands::run::process_services,
    db::{db, types::Run},
};

pub fn rollback(app: &App, args: &RollbackArgs) -> Result<()> {
    let conn = db(app)?;

    if let Ok(last_run) = Run::get_previous(&conn) {
        let run_args = RunArgs {
            dry_run: false,
            noconfirm: args.noconfirm,
        };

        process_services(app, &run_args, &last_run.data)?;

        last_run.delete(&conn)?;
    } else {
        bail!("No previous run to rollback to")
    }

    Ok(())
}
