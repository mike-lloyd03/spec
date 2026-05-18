use anyhow::Result;
use serde::{Deserialize, Serialize};
use toml::Table;

use crate::{
    adapters::ini::IniAdapter,
    types::{
        file_artifact::FileArtifact,
        managed_service::{ManagedService, Plan, ServiceConfig, ServiceState},
        paths::Paths,
    },
};

pub struct AutoCpuFreqService;

#[derive(Serialize, Deserialize, Default)]
struct AutoCpuFreqConfig {
    pub charger: Option<PowerConfig>,
    pub battery: Option<PowerConfig>,
    #[serde(skip_serializing)]
    pub service: Option<ServiceConfig>,
}

#[derive(Serialize, Deserialize, Default)]
struct PowerConfig {
    pub governor: Option<Governor>,
    pub energy_performance_preference: Option<Epp>,
    pub turbo: Option<TurboMode>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
enum Governor {
    Performance,
    #[default]
    Powersave,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
enum Epp {
    #[default]
    Default,
    Performance,
    BalancePerformance,
    BalancePower,
    Power,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
enum TurboMode {
    Always,
    #[default]
    Auto,
    Never,
}

impl ManagedService for AutoCpuFreqService {
    fn name(&self) -> &str {
        "auto-cpufreq"
    }

    fn plan(&self, config_table: &Table, paths: &Paths) -> Result<Plan> {
        let config: AutoCpuFreqConfig = self.parse_config(config_table)?;

        let mut adapter = IniAdapter::new();
        let content = adapter
            .spaces_around_equals(true)
            .push_line("# Managed by spec")
            .section("charger", config.charger)?
            .section("battery", config.battery)?
            .build();

        let files = vec![FileArtifact {
            path: paths.system_config.join("auto-cpufreq.conf"),
            content,
            permissions: 0o644,
            requires_root: true,
        }];

        let service_state = config
            .service
            .map(|state| ServiceState::new(self.name(), state.enabled, state.running));

        Ok(Plan {
            files,
            service_state,
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use anyhow::Result;

    #[test]
    fn test_plan() -> Result<()> {
        let config_table = Table::from_str(
            r#"
[charger]
governor = "performance"
energy_performance_preference = "performance"
turbo = "auto"

[battery]
energy_performance_preference = "balance_power"
turbo = "never"
        "#,
        )?;

        let service = AutoCpuFreqService;
        let paths = Paths {
            system_config: "/test".into(),
            ..Default::default()
        };

        let plan = service.plan(&config_table, &paths)?;

        assert_eq!(plan.files.len(), 1);

        if let Some(file) = plan.files.first() {
            assert_eq!(
                file.path.to_string_lossy().to_string(),
                "/test/auto-cpufreq.conf".to_string()
            );

            let expected_content = r#"# Managed by spec
[charger]
energy_performance_preference = performance
governor = performance
turbo = auto

[battery]
energy_performance_preference = balance_power
turbo = never
"#;

            assert_eq!(expected_content.trim(), file.content.trim());
        }

        Ok(())
    }
}
