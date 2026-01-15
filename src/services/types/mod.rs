use anyhow::{Context, Result};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};
use toml::Table;

mod services_config;
pub use services_config::*;

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

pub trait ManagedService {
    fn name(&self) -> &str;

    fn plan(
        &self,
        config: &Table,
        sys_config_dir: &Path,
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
