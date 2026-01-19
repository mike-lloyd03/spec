use serde::{Deserialize, Serialize};
use toml::Table;

use crate::{
    adapters::ini::IniAdapter,
    types::{
        managed_service::{FileArtifact, ManagedService, ServiceConfig, ServiceState},
        paths::Paths,
    },
};

pub struct TuneDPPDService;

#[derive(Serialize, Deserialize, Default)]
struct TuneDPPDConfig {
    pub main: Option<Main>,
    pub profiles: Option<Profiles>,
    pub battery: Option<Battery>,

    #[serde(skip_serializing)]
    pub service: Option<ServiceConfig>,
}

#[derive(Serialize, Deserialize, Default)]
struct Main {
    pub default: Option<String>,
    pub battery_detection: Option<bool>,
    pub sysfs_acpi_monitor: Option<bool>,
}

#[derive(Serialize, Deserialize, Default)]
struct Profiles {
    #[serde(rename(serialize = "power-saver"))]
    pub power_saver: Option<String>,
    pub balanced: Option<String>,
    pub performance: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
struct Battery {
    pub balanced: Option<String>,
}

impl ManagedService for TuneDPPDService {
    fn name(&self) -> &str {
        "tuned"
    }

    fn plan(
        &self,
        config: &Table,
        paths: &Paths,
    ) -> anyhow::Result<(Vec<FileArtifact>, Option<ServiceState>)> {
        let config: TuneDPPDConfig = self.parse_config(config)?;

        let mut adapter = IniAdapter::new();
        let content = adapter
            .push_line("# Managed by spec")
            .section("main", config.main)?
            .section("profiles", config.profiles)?
            .section("battery", config.battery)?
            .build();

        let file = FileArtifact {
            path: paths.system_config.join("tuned/ppd.conf"),
            content,
            permissions: 0o644,
        };

        let service_state = config.service.map(|state| ServiceState {
            name: self.name().to_string(),
            enabled: state.enabled,
            running: state.running,
            ..Default::default()
        });

        Ok((vec![file], service_state))
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
[main]
default = "balanced"
battery_detection = true
sysfs_acpi_monitor = true

[profiles]
power_saver = "powersave"
balanced = "balanced"
performance = "throughput-performance"

[battery]
balanced = "balanced-battery"
        "#,
        )?;

        let service = TuneDPPDService;
        let paths = Paths {
            system_config: "/test".into(),
            ..Default::default()
        };

        let (files, _) = service.plan(&config_table, &paths)?;

        assert_eq!(files.len(), 1);

        if let Some(file) = files.first() {
            assert_eq!(
                file.path.to_string_lossy().to_string(),
                "/test/tuned/ppd.conf".to_string()
            );

            let expected_content = r#"# Managed by spec
[main]
battery_detection=true
default=balanced
sysfs_acpi_monitor=true

[profiles]
balanced=balanced
performance=throughput-performance
power-saver=powersave

[battery]
balanced=balanced-battery"#;

            assert_eq!(expected_content.trim(), file.content.trim());
        }

        Ok(())
    }
}
