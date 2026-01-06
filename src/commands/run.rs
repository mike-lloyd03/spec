use cliclack::{confirm, intro, log, note, outro};
use color_eyre::{
    Result,
    eyre::{Context, bail},
};
use sha2::{Digest, Sha256};
use std::{fs, io::Write};
use toml::Table;

use crate::{
    cli::RunArgs,
    services::{FileArtifact, ManagedService, ServiceState, get_service_by_name},
};

pub fn run(args: &RunArgs) -> Result<()> {
    let config_dir = xdg::BaseDirectories::new()
        .get_config_home()
        .expect("User HOME should exist")
        .join("tenant")
        .join("config.toml");

    let config_file_path = config_dir
        .to_str()
        .expect("config dir should be unicode")
        .to_owned();

    let content = fs::read_to_string(config_dir).context(format!(
        "Failed to open config file at '{config_file_path}'",
    ))?;

    let root: Table = toml::from_str(&content)?;

    for (key, value) in root {
        if let Some(table) = value.as_table() {
            if let Some(service) = get_service_by_name(&key) {
                intro(format!("Service: {}", key))?;
                apply_service(service, table, args)?;
                outro("\n")?;
            } else {
                log::error(format!("Unknown service section: {}", key))?;
            }
        }
    }

    Ok(())
}

fn apply_service(service: Box<dyn ManagedService>, config: &Table, args: &RunArgs) -> Result<()> {
    let (files, service_state) = service.plan(config, &args.sys_config_dir)?;
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
