use crate::{
    db::connect_db,
    types::{managed_services_config::ManagedServicesConfig, paths::Paths},
    utils::string_from_env_or_default,
};
use anyhow::Result;
use rusqlite::Connection;

pub struct App {
    pub db: Connection,
    pub paths: Paths,
    pub systemctl_cmd: String,
    pub managed_services: ManagedServicesConfig,
}

impl App {
    pub fn new() -> Result<Self> {
        let paths = Paths::load()?;

        let systemctl_cmd = string_from_env_or_default("SPEC_SYSTEMCTL_CMD", "systemctl")?;

        let db = connect_db(&paths.spec_state)?;

        let managed_services = ManagedServicesConfig::load(&paths.spec_config)?;

        Ok(Self {
            db,
            paths,
            systemctl_cmd,
            managed_services,
        })
    }
}
