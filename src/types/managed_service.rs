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

pub struct ServiceState {
    pub name: String,
    pub enabled: Option<bool>,
    pub running: Option<bool>,
    pub reload_cmd: String,
    pub start_cmd: String,
    pub enable_cmd: String,
    pub stop_cmd: String,
    pub disable_cmd: String,
    pub requires_sudo: bool,
}

impl ServiceState {
    pub fn new(name: &str, enabled: Option<bool>, running: Option<bool>) -> Self {
        Self {
            name: name.to_string(),
            enabled,
            running,
            reload_cmd: format!("systemctl reload {name}"),
            start_cmd: format!("systemctl start {name}"),
            enable_cmd: format!("systemctl enable {name}"),
            stop_cmd: format!("systemctl stop {name}"),
            disable_cmd: format!("systemctl disable {name}"),
            requires_sudo: true,
        }
    }

    pub fn builder(name: &str) -> Self {
        Self::new(name, None, None)
    }

    pub fn enabled(mut self, enabled: Option<bool>) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn running(mut self, running: Option<bool>) -> Self {
        self.running = running;
        self
    }

    pub fn reload_cmd(mut self, reload_cmd: &str) -> Self {
        self.reload_cmd = reload_cmd.to_string();
        self
    }

    pub fn start_cmd(mut self, start_cmd: &str) -> Self {
        self.start_cmd = start_cmd.to_string();
        self
    }

    pub fn enable_cmd(mut self, enable_cmd: &str) -> Self {
        self.enable_cmd = enable_cmd.to_string();
        self
    }

    pub fn stop_cmd(mut self, stop_cmd: &str) -> Self {
        self.stop_cmd = stop_cmd.to_string();
        self
    }

    pub fn disable_cmd(mut self, disable_cmd: &str) -> Self {
        self.disable_cmd = disable_cmd.to_string();
        self
    }

    pub fn requires_sudo(mut self, requires_sudo: bool) -> Self {
        self.requires_sudo = requires_sudo;
        self
    }
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
