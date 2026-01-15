use anyhow::Result;
use serde::{Deserialize, Serialize};
use toml::Table;

use crate::types::{
    managed_service::{FileArtifact, ManagedService, ServiceConfig, ServiceState},
    paths::Paths,
};

pub struct AutoCpuFreqService;

#[derive(Serialize, Deserialize, Default)]
struct AutoCpuFreqConfig {
    #[serde(skip_serializing)]
    pub service: Option<ServiceConfig>,
}

impl ManagedService for AutoCpuFreqService {
    fn name(&self) -> &str {
        "auto-cpufreq"
    }

    fn plan(
        &self,
        config_table: &Table,
        _: &Paths,
    ) -> Result<(Vec<FileArtifact>, Option<ServiceState>)> {
        let config: AutoCpuFreqConfig = self.parse_config(config_table)?;

        let service_state = config.service.map(|state| ServiceState {
            name: self.name().to_string(),
            enabled: state.enabled.unwrap_or_default(),
            running: state.running.unwrap_or_default(),
        });

        Ok((vec![], service_state))
    }
}
