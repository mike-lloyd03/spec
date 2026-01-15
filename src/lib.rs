pub mod adapters;
pub mod cli;
pub mod commands;
pub mod db;
pub mod services;
mod utils;

use crate::{db::connect_db, services::types::ManagedServices};
use anyhow::Result;
use rusqlite::Connection;
use std::{env, fs, path::PathBuf, str::FromStr};

pub struct App {
    pub db: Connection,
    pub config_dir: PathBuf,
    pub state_dir: PathBuf,
    pub system_config_dir: PathBuf,
    pub systemctl_cmd: String,
    pub managed_services: ManagedServices,
}

impl App {
    pub fn new() -> Result<Self> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("spec");

        let config_dir = dir_from_env_or_default("SPEC_CONFIG_DIR", xdg_dirs.config_home.unwrap())?;

        let state_dir = dir_from_env_or_default("SPEC_STATE_DIR", xdg_dirs.state_home.unwrap())?;
        fs::create_dir_all(&state_dir)?;

        let system_config_dir =
            dir_from_env_or_default("SPEC_SYSTEM_CONFIG_DIR", PathBuf::from_str("/env")?)?;

        let systemctl_cmd = string_from_env_or_default("SPEC_SYSTEMCTL_CMD", "systemctl")?;

        let db = connect_db(&state_dir)?;

        let managed_services = ManagedServices::load(&config_dir)?;

        Ok(Self {
            db,
            config_dir,
            state_dir,
            system_config_dir,
            systemctl_cmd,
            managed_services,
        })
    }
}

fn dir_from_env_or_default(var_name: &str, default: PathBuf) -> Result<PathBuf> {
    Ok(if let Ok(dir) = env::var(var_name) {
        PathBuf::from_str(&dir)?
    } else {
        default
    })
}

fn string_from_env_or_default(var_name: &str, default: &str) -> Result<String> {
    Ok(if let Ok(s) = env::var(var_name) {
        s
    } else {
        default.to_string()
    })
}
