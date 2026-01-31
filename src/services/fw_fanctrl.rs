use anyhow::Result;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use toml::Table;

use crate::types::{
    file_artifact::FileArtifact,
    managed_service::{ManagedService, Plan, ServiceConfig, ServiceState},
    paths::Paths,
};

pub struct FwFanCtrlService;

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all(serialize = "camelCase"))]
struct FwFanCtrlConfig {
    default_strategy: String,
    strategy_on_discharging: String,
    strategies: IndexMap<String, Strategy>,

    #[serde(skip_serializing)]
    pub service: Option<ServiceConfig>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all(serialize = "camelCase"))]
struct Strategy {
    fan_speed_update_frequency: u32,
    moving_average_interval: u32,
    speed_curve: Vec<SpeedCurvePoint>,
}

#[derive(Serialize, Deserialize, Default)]
struct SpeedCurvePoint {
    temp: u32,
    speed: u32,
}

impl ManagedService for FwFanCtrlService {
    fn name(&self) -> &str {
        "fw-fanctrl"
    }

    fn plan(&self, config: &Table, paths: &Paths) -> Result<Plan> {
        let config: FwFanCtrlConfig = self.parse_config(config)?;

        let content = serde_json::to_string(&config)?;

        let files = vec![FileArtifact {
            path: paths.system_config.join("fw-fanctrl/config.json"),
            content,
            permissions: 0o644,
            requires_root: true,
        }];

        let service_state = config
            .service
            .map(|s| ServiceState::new(self.name(), s.enabled, s.running));

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
default_strategy = "medium"
strategy_on_discharging = ""

[strategies.lazy]
fan_speed_update_frequency = 5
moving_average_interval = 30
speed_curve = [{temp = 0, speed = 15}, {temp = 50, speed = 15}]
        "#,
        )?;

        let service = FwFanCtrlService;
        let paths = Paths {
            system_config: "/test".into(),
            ..Default::default()
        };

        let plan = service.plan(&config_table, &paths)?;

        assert_eq!(plan.files.len(), 1);

        if let Some(file) = plan.files.first() {
            assert_eq!(
                file.path.to_string_lossy().to_string(),
                "/test/fw-fanctrl/config.json".to_string()
            );

            let expected_content = r#"{
    "defaultStrategy": "medium",
    "strategyOnDischarging" : "",
    "strategies": {
        "lazy": {
            "fanSpeedUpdateFrequency": 5,
            "movingAverageInterval": 30,
            "speedCurve": [
                { "temp": 0, "speed": 15 },
                { "temp": 50, "speed": 15 }
            ]
        }
    }
}"#;

            let expected_json: serde_json::Value = serde_json::from_str(expected_content)?;
            let got_json: serde_json::Value = serde_json::from_str(&file.content)?;

            assert_eq!(expected_json, got_json);
        }

        Ok(())
    }
}
