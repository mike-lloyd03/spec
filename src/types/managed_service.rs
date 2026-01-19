use anyhow::{Context, Result};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::path::PathBuf;
use toml::Table;

use crate::types::paths::Paths;
pub struct FileArtifact {
    pub path: PathBuf,
    pub content: String,
    pub permissions: u32,
}

#[derive(Default)]
pub struct ServiceState {
    pub name: String,
    pub enabled: Option<bool>,
    pub running: Option<bool>,
    pub reload_cmd: Option<String>,
}

#[derive(Default, Deserialize)]
pub struct ServiceConfig {
    pub enabled: Option<bool>,
    pub running: Option<bool>,
}

pub trait ManagedService {
    fn name(&self) -> &str;

    fn plan(
        &self,
        config: &Table,
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
