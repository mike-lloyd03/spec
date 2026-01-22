use anyhow::Result;
use serde::{Deserialize, Serialize};
use toml::Table;

use crate::types::{
    managed_service::{ManagedService, Plan, ServiceConfig, ServiceState},
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

    fn plan(&self, config_table: &Table, _: &Paths) -> Result<Plan> {
        let config: AutoCpuFreqConfig = self.parse_config(config_table)?;

        let service_state = config
            .service
            .map(|state| ServiceState::new(self.name(), state.enabled, state.running));

        Ok(Plan {
            service_state,
            ..Default::default()
        })
    }
}
