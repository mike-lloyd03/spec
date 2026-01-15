use anyhow::{Context, Result};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::path::PathBuf;
use toml::Table;

use crate::types::managed_services_config::ConfigScope;
use crate::types::paths::Paths;
pub struct FileArtifact {
    pub path: PathBuf,
    pub content: String,
    pub permissions: u32,
}

#[derive(Default)]
pub struct ServiceState {
    pub name: String,
    pub enabled: bool,
    pub running: bool,
}

#[derive(Default, Deserialize)]
pub struct ServiceConfig {
    pub enabled: Option<bool>,
    pub running: Option<bool>,
}
pub enum ManagedServiceCapability {
    User,
    System,
    UserAndSystem,
}

impl ManagedServiceCapability {
    pub fn allows(&self, scope: ConfigScope) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match (self, scope) {
            (ManagedServiceCapability::User, ConfigScope::User) => true,
            (ManagedServiceCapability::System, ConfigScope::System) => true,
            (ManagedServiceCapability::UserAndSystem, _) => true,
            _ => false,
        }
    }
}

pub trait ManagedService {
    fn name(&self) -> &str;

    fn capabilities(&self) -> ManagedServiceCapability {
        ManagedServiceCapability::System
    }

    fn plan(
        &self,
        config: &Table,
        config_scope: ConfigScope,
        paths: &Paths,
    ) -> Result<(Vec<FileArtifact>, Option<ServiceState>)>;

    fn parse_config<T: DeserializeOwned>(&self, config: &Table) -> Result<T>
    where
        Self: Sized,
    {
        config
            .clone()
            .try_into()
            .context(format!("while reading '{}' section", self.name()))
    }
}
