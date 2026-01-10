use std::path::Path;

use color_eyre::Result;
use serde::{Deserialize, Serialize};
use toml::Table;

use crate::services::{FileArtifact, ManagedService, ServiceConfig, ServiceState};

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
        _: &Path,
    ) -> Result<(Vec<FileArtifact>, Option<ServiceState>)> {
        let config: AutoCpuFreqConfig = self.parse_config(config_table)?;

        let service_state = if let Some(state) = config.service {
            ServiceState {
                name: self.name().to_string(),
                enabled: state.enabled.unwrap_or_default(),
                running: state.running.unwrap_or_default(),
            }
        } else {
            ServiceState::default()
        };

        Ok((vec![], Some(service_state)))
    }
}
