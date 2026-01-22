use anyhow::Result;
use cliclack::{
    confirm, intro,
    log::{self, error, info, warning},
    note, outro,
};
use colored::Colorize;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    process::{Command, Output},
};
use toml::Table;

use crate::{
    services::get_service_by_name,
    types::{
        app::App,
        cli::RunArgs,
        file_artifact::FileArtifact,
        managed_service::{ManagedService, ServiceState},
        managed_services_config::ManagedServicesConfig,
        run::Run,
    },
    utils::{bytes_to_string, hash_bytes},
};

pub fn run(app: &App, args: &RunArgs) -> Result<()> {
    let previous_run = Run::get_previous(&app.db);

    if let Ok(last_run) = &previous_run
        && app.managed_services == last_run.data
    {
        warning("No changes from previous run")?;
        return Ok(());
    }

    let mut managed_files = vec![];
    process_services(app, args, &app.managed_services, &mut managed_files)?;

    if !args.dry_run {
        let mut new_run = Run::new(app.managed_services.clone(), &app.paths.system_config);
        new_run.managed_files = managed_files.clone();
        new_run.create(&app.db)?;

        if let Ok(last_run) = previous_run {
            rm_old_files(&managed_files, &last_run.managed_files)?;
        }
    }

    Ok(())
}

pub fn process_services(
    app: &App,
    args: &RunArgs,
    services: &ManagedServicesConfig,
    managed_files: &mut Vec<String>,
) -> Result<()> {
    for (key, value) in &services.data {
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
    let plan = service.plan(config, &app.paths)?;
    let mut needs_reload = false;

    for file in plan.files {
        managed_files.push(file.path.display().to_string());

        if ensure_file(&file, args)? {
            log::warning(format!("File {} [Changed]", file.path.display()))?;
            needs_reload = true;
        } else {
            log::step(format!("File {} [OK]", file.path.display()))?;
        }
    }

    if let Some(state) = plan.service_state {
        apply_systemd(state, needs_reload, args)?;
    }

    if let Some(states) = plan.addl_service_states {
        for state in states {
            apply_systemd(state, false, args)?;
        }
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

fn create_file(file: &FileArtifact, args: &RunArgs) -> Result<bool> {
    note(file.path.to_str().unwrap_or_default(), file.content.clone())?;

    let prompt_text = {
        let path_display = file.path.display().to_string().blue();

        if file.path.exists() {
            format!("{path_display} has changed. Overwrite?")
        } else {
            format!("Create file {path_display}?")
        }
    };

    let should_continue;

    if args.noconfirm {
        should_continue = true;
        log::step(prompt_text)?;
    } else {
        should_continue = confirm(prompt_text).initial_value(true).interact()?;
    }

    if should_continue && !args.dry_run {
        file.write()?;
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
            let output = cmd_from_str(&state.reload_cmd, state.requires_root)?;

            if !output.status.success() {
                error(format!(
                    "Error reloading service: {} {}",
                    bytes_to_string(&output.stdout)?,
                    bytes_to_string(&output.stderr)?
                ))?;
            }
        }
    }

    if !args.dry_run {
        if let Some(running) = state.running {
            let cmd = match running {
                true => state.start_cmd,
                false => state.stop_cmd,
            };

            cmd_from_str(&cmd, state.requires_root)?;
        }

        if let Some(enabled) = state.enabled {
            let cmd = match enabled {
                true => state.enable_cmd,
                false => state.disable_cmd,
            };

            cmd_from_str(&cmd, state.requires_root)?;
        }
    }

    log::info(format!(
        "[Service] Ensure {} is enabled={} active={}",
        state.name,
        render_system_state(state.enabled),
        render_system_state(state.running)
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

fn render_system_state(val: Option<bool>) -> String {
    match val {
        Some(true) => "true",
        Some(false) => "false",
        None => "ignored",
    }
    .to_string()
}

fn cmd_from_str(cmd_str: &str, requires_sudo: bool) -> Result<Output> {
    // println!("Running: ${cmd_str} with sudo: {requires_sudo}");
    Ok(match (cmd_str.split_once(" "), requires_sudo) {
        (Some((cmd, args)), true) => Command::new("sudo")
            .arg(cmd)
            .args(args.split(" "))
            .output()?,
        (Some((cmd, args)), false) => Command::new(cmd).args(args.split(" ")).output()?,
        (None, true) => Command::new("sudo").arg(cmd_str).output()?,
        (None, false) => Command::new(cmd_str).output()?,
    })
}
