use crate::{
    db::connect_db,
    types::{managed_services_config::ManagedServicesConfig, paths::Paths},
};
use anyhow::Result;
use rusqlite::Connection;

pub struct App {
    pub db: Connection,
    pub paths: Paths,
    pub managed_services: ManagedServicesConfig,
}

impl App {
    pub fn new() -> Result<Self> {
        let paths = Paths::load()?;

        let db = connect_db(&paths.spec_state)?;

        let managed_services = ManagedServicesConfig::load(&paths.spec_config)?;

        Ok(Self {
            db,
            paths,
            managed_services,
        })
    }
}
