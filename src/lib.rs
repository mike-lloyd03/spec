pub mod adapters;
pub mod cli;
pub mod commands;
pub mod db;
pub mod services;
mod utils;

use anyhow::Result;
use rusqlite::Connection;
use std::{env, fs, path::PathBuf, str::FromStr};
use toml::Table;

use crate::db::connect_db;

pub struct App {
    pub db: Connection,
    pub config_dir: PathBuf,
    pub state_dir: PathBuf,
    pub system_config_dir: PathBuf,
    pub systemctl_cmd: String,
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

        Ok(Self {
            db,
            config_dir,
            state_dir,
            system_config_dir,
            systemctl_cmd,
        })
    }

    pub fn load_services(&self) -> Result<Table> {
        let mut merged = Table::new();

        let services_dir = self.config_dir.join("services");
        if services_dir.exists() {
            for entry in fs::read_dir(services_dir)? {
                let path = entry?.path();

                if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    let content = fs::read_to_string(&path)?;
                    let table: Table = toml::from_str(&content)?;

                    merge_tables(&mut merged, table);
                }
            }
        }

        Ok(merged)
    }
}

fn merge_tables(base: &mut Table, incoming: Table) {
    for (key, value) in incoming {
        base.insert(key, value);
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
