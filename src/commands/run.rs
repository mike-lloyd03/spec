use anyhow::{Result, bail};
use cliclack::{
    confirm, intro,
    log::{self, error, info, warning},
    note, outro,
};
use std::{fs, io::Write, os::unix::fs::PermissionsExt};
use toml::Table;

use crate::{
    App,
    cli::RunArgs,
    db::types::Run,
    services::{
        get_service_by_name,
        types::{FileArtifact, ManagedService, ServiceState},
    },
    utils::{bytes_to_string, hash_bytes},
};

pub fn run(app: &App, args: &RunArgs) -> Result<()> {
    let services = app.load_services()?;
    let previous_run = Run::get_previous(&app.db);

    if let Ok(last_run) = &previous_run
        && services == last_run.data
    {
        warning("No changes from previous run")?;
        return Ok(());
    }

    let mut managed_files = vec![];
    process_services(app, args, &services, &mut managed_files)?;

    let mut new_run = Run::new(services.clone(), &app.system_config_dir);
    new_run.managed_files = managed_files.clone();
    new_run.create(&app.db)?;

    if let Ok(last_run) = previous_run {
        rm_old_files(&managed_files, &last_run.managed_files)?;
    }

    Ok(())
}

pub fn process_services(
    app: &App,
    args: &RunArgs,
    services: &Table,
    managed_files: &mut Vec<String>,
) -> Result<()> {
    println!("Running data: {:?}", services);
    for (key, value) in services {
        if let Some(table) = value.as_table() {
            if let Some(service) = get_service_by_name(key) {
                intro(format!("Service: {}", key))?;
                apply_service(app, service, table, args, managed_files)?;
                outro("\n")?;
            } else {
                log::error(format!("Unknown service section: {}", key))?;
            }
        }
    }

    Ok(())
}

fn apply_service(
    app: &App,
    service: Box<dyn ManagedService>,
    config: &Table,
    args: &RunArgs,
    managed_files: &mut Vec<String>,
) -> Result<()> {
    let (files, service_state) = service.plan(config, &app.system_config_dir)?;
    let mut needs_reload = false;

    for file in files {
        managed_files.push(
            file.path
                .to_str()
                .expect("PathBuf should convert")
                .to_owned(),
        );

        if ensure_file(&file, args)? {
            log::warning(format!("File {} [Changed]", file.path.display()))?;
            needs_reload = true;
        } else {
            log::step(format!("File {} [OK]", file.path.display()))?;
        }
    }

    if let Some(state) = service_state {
        apply_systemd(app, state, needs_reload, args)?;
    }

    Ok(())
}

fn ensure_file(artifact: &FileArtifact, args: &RunArgs) -> Result<bool> {
    let new_hash = hash_bytes(artifact.content.as_bytes());

    let current_hash = if artifact.path.exists() {
        let bytes = fs::read(&artifact.path)?;
        Some(hash_bytes(&bytes))
    } else {
        None
    };

    if Some(new_hash) != current_hash {
        create_file(artifact, args)
    } else {
        check_and_fix_permissions(artifact, args)
    }
}

fn create_file(artifact: &FileArtifact, args: &RunArgs) -> Result<bool> {
    note(
        artifact.path.to_str().unwrap_or_default(),
        artifact.content.clone(),
    )?;

    let prompt_text = format!(
        "{} has changed. Overwrite?",
        artifact.path.to_str().unwrap_or("unknown")
    );

    let should_continue;

    if args.noconfirm {
        should_continue = true;
        log::step(prompt_text)?;
    } else {
        should_continue = confirm(prompt_text).initial_value(true).interact()?;
    }

    if should_continue && !args.dry_run {
        if let Some(parent_dir) = artifact.path.parent() {
            fs::DirBuilder::new().recursive(true).create(parent_dir)?;
        } else {
            bail!(
                "Parent directory for {} does not exist",
                artifact.path.to_str().expect("")
            );
        }

        let mut file = fs::File::create(&artifact.path)?;
        file.write_all(artifact.content.as_bytes())?;

        let perms = fs::Permissions::from_mode(artifact.permissions);
        file.set_permissions(perms)?;
    }
    Ok(true)
}

fn check_and_fix_permissions(artifact: &FileArtifact, args: &RunArgs) -> Result<bool> {
    let target_perms = fs::Permissions::from_mode(artifact.permissions);
    let existing_perms = fs::metadata(&artifact.path)?.permissions();

    if existing_perms.mode() & 0o777 != target_perms.mode() {
        let prompt_text = format!(
            "Permissions are incorrect for {}. (Are {:o} should be {:o}).\nCorrect them?",
            artifact.path.to_str().unwrap_or_default(),
            existing_perms.mode() & 0o777,
            target_perms.mode()
        );

        let should_continue;

        if args.noconfirm {
            should_continue = true;
            log::step(prompt_text)?;
        } else {
            should_continue = confirm(prompt_text).initial_value(true).interact()?;
        }

        if should_continue {
            fs::set_permissions(&artifact.path, target_perms)?;
        }
        return Ok(true);
    }
    Ok(false)
}

fn apply_systemd(app: &App, state: ServiceState, needs_reload: bool, args: &RunArgs) -> Result<()> {
    if needs_reload {
        log::step(format!("Reload service: {}", state.name))?;

        if !args.dry_run {
            let output = std::process::Command::new(&app.systemctl_cmd)
                .arg("reload")
                .output()?;

            if !output.status.success() {
                error(format!(
                    "Error reloading service: {} {}",
                    bytes_to_string(&output.stdout)?,
                    bytes_to_string(&output.stderr)?
                ))?;
            }
        }
    }

    log::info(format!(
        "[Service] Ensure {} is enabled={} active={}",
        state.name, state.enabled, state.running
    ))?;
    Ok(())
}

fn rm_old_files(new_run_files: &[String], prev_run_files: &[String]) -> Result<()> {
    for filepath in prev_run_files {
        println!("Old file: {filepath}");
        if !new_run_files.contains(filepath) {
            info(format!("Removing orphaned file: {}", filepath))?;
            fs::remove_file(filepath)?;
        }
    }
    Ok(())
}
