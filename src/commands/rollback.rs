use color_eyre::eyre::Result;

use crate::{
    cli::{RollbackArgs, RunArgs},
    commands::run::{process_services, save_run},
    db::{db, types::Run},
};

pub fn rollback(args: &RollbackArgs) -> Result<()> {
    let conn = db()?;

    if let Ok(last_run) = Run::get_previous(&conn) {
        let run_args = RunArgs {
            dry_run: false,
            sys_config_dir: last_run.sys_config_dir,
            noconfirm: false,
        };

        process_services(&run_args, &last_run.data)?;

        save_run(&run_args, &conn, last_run.data)?;
    }

    Ok(())
}
