use color_eyre::{Result, eyre::Context};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::path::PathBuf;
use toml::Table;

use crate::services::autocpu_freq::AutoCpuFreqService;
use crate::services::ssh::SshService;
use crate::services::sshd::SshdService;

mod autocpu_freq;
mod ssh;
mod sshd;

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

    fn plan(&self, config: &Table) -> Result<(Vec<FileArtifact>, Option<ServiceState>)>;

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

pub fn get_service_by_name(name: &str) -> Option<Box<dyn ManagedService>> {
    match name {
        "auto-cpufreq" => Some(Box::new(AutoCpuFreqService)),
        "ssh" => Some(Box::new(SshService)),
        "sshd" => Some(Box::new(SshdService)),
        _ => None,
    }
}
