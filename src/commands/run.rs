use cliclack::{confirm, intro, log, note, outro};
use color_eyre::{Result, eyre::bail};
use sha2::{Digest, Sha256};
use std::{fs, io::Write, os::unix::fs::PermissionsExt};
use toml::Table;

use crate::{
    App,
    cli::RunArgs,
    db::types::Run,
    services::{FileArtifact, ManagedService, ServiceState, get_service_by_name},
};

pub fn run(app: &App, args: &RunArgs) -> Result<()> {
    let services = app.load_services()?;

    process_services(app, args, &services)?;

    if let Ok(last_run) = Run::get_previous(&app.db)
        && services == last_run.data
    {
    } else {
        save_run(app, services)?;
    }

    Ok(())
}

pub fn process_services(app: &App, args: &RunArgs, services: &Table) -> Result<()> {
    println!("Running data: {:?}", services);
    for (key, value) in services {
        if let Some(table) = value.as_table() {
            if let Some(service) = get_service_by_name(key) {
                intro(format!("Service: {}", key))?;
                apply_service(app, service, table, args)?;
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
) -> Result<()> {
    let (files, service_state) = service.plan(config, app.system_config_dir.to_str().unwrap())?;
    let mut needs_reload = false;

    for file in files {
        if ensure_file(&file, args)? {
            log::warning(format!("File {} [Changed]", file.path.display()))?;
            needs_reload = true;
        } else {
            log::step(format!("File {} [OK]", file.path.display()))?;
        }
    }

    if let Some(state) = service_state {
        apply_systemd(state, needs_reload, args)?;
    }

    Ok(())
}

fn ensure_file(artifact: &FileArtifact, args: &RunArgs) -> Result<bool> {
    let mut hasher = Sha256::new();
    hasher.update(artifact.content.as_bytes());
    let new_hash = hex::encode(hasher.finalize());

    let current_hash = if artifact.path.exists() {
        let bytes = fs::read(&artifact.path)?;
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        Some(hex::encode(hasher.finalize()))
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

fn apply_systemd(state: ServiceState, needs_reload: bool, args: &RunArgs) -> Result<()> {
    if needs_reload {
        log::step(format!("Reload service: {}", state.name))?;
        if !args.dry_run {
            // std::process::Command::new("systemctl").arg("reload")...
        }
    }

    log::info(format!(
        "[Service] Ensure {} is enabled={} active={}",
        state.name, state.enabled, state.running
    ))?;
    Ok(())
}

pub fn save_run(app: &App, service_config: Table) -> Result<()> {
    let service_table = Table::from(service_config.clone());
    let run = Run::new(
        service_table,
        app.system_config_dir.to_str().unwrap().to_string(),
    );
    run.create(&app.db)?;
    Ok(())
}
