mod adapters;
mod services;

use color_eyre::{Result, eyre::Context};
use services::get_service_by_name;
use sha2::{Digest, Sha256};
use std::fs;
use toml::Table;

fn main() -> Result<()> {
    color_eyre::install()?;

    let config_dir = xdg::BaseDirectories::new()
        .get_config_home()
        .expect("User HOME should exist")
        .join("tenant")
        .join("config.toml");

    let config_file_path = config_dir
        .to_str()
        .expect("config dir should be unicode")
        .to_owned();

    println!("Reading configuration from {config_file_path}");

    let content = fs::read_to_string(config_dir).context(format!(
        "Failed to open config file at '{config_file_path}'",
    ))?;

    let root: Table = toml::from_str(&content)?;

    for (key, value) in root {
        if let Some(table) = value.as_table() {
            if let Some(service) = get_service_by_name(&key) {
                println!("\n--> Processing service: {}", key);
                apply_service(service, table)?;
            } else {
                println!("?? Unknown service section: {}", key);
            }
        }
    }

    Ok(())
}

fn apply_service(service: Box<dyn services::ManagedService>, config: &Table) -> Result<()> {
    let (files, service_state) = service.plan(config)?;
    let mut needs_reload = false;

    for file in files {
        if ensure_file(&file)? {
            println!("[Changed] {}", file.path.display());
            needs_reload = true;
        } else {
            println!("[OK] {}", file.path.display());
        }
    }

    if let Some(state) = service_state {
        apply_systemd(state, needs_reload)?;
    }

    Ok(())
}

fn ensure_file(artifact: &services::FileArtifact) -> Result<bool> {
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
        println!(
            "Write to {:?} (Permissions: {:o})",
            artifact.path, artifact.permissions
        );
        println!("Content:\n------\n{}\n------", artifact.content);
        return Ok(true);
    }

    Ok(false)
}

fn apply_systemd(state: services::ServiceState, needs_reload: bool) -> Result<()> {
    if needs_reload {
        println!("Reload service: {}", state.name);
        // std::process::Command::new("systemctl").arg("reload")...
    }

    println!(
        "[Service] Ensure {} is enabled={} active={}",
        state.name, state.enabled, state.running
    );
    Ok(())
}
