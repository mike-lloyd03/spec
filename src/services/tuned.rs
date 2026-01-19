use serde::{Deserialize, Serialize};
use toml::Table;

use crate::{
    adapters::key_value::{BoolStyle, KeyValueAdapter},
    types::{
        managed_service::{FileArtifact, ManagedService, ServiceConfig, ServiceState},
        paths::Paths,
    },
};

pub struct TuneDService;

#[derive(Serialize, Deserialize, Default)]
struct TuneDConfig {
    pub daemon: bool,
    pub default_instance_priority: u32,
    pub dynamic_tuning: bool,
    pub reapply_sysctl: bool,
    pub recommend_command: bool,
    pub sleep_interval: u32,
    pub update_interval: u32,

    #[serde(skip_serializing)]
    pub service: Option<ServiceConfig>,
}

impl ManagedService for TuneDService {
    fn name(&self) -> &str {
        "tuned"
    }

    fn plan(
        &self,
        config: &Table,
        paths: &Paths,
    ) -> anyhow::Result<(Vec<FileArtifact>, Option<ServiceState>)> {
        let config: TuneDConfig = self.parse_config(config)?;

        let mut adapter = KeyValueAdapter::new(" = ", "#").bool_style(BoolStyle::OneZero);

        adapter.comment("Managed by spec");
        adapter.parse_struct(&config)?;

        let file = FileArtifact {
            path: paths.system_config.join("tuned/tuned-main.conf"),
            content: adapter.build(),
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
daemon = true
dynamic_tuning = true
sleep_interval = 1
update_interval = 10
recommend_command = false
reapply_sysctl = true
default_instance_priority = 0
        "#,
        )?;

        let service = TuneDService;
        let paths = Paths {
            system_config: "/test".into(),
            ..Default::default()
        };

        let (files, _) = service.plan(&config_table, &paths)?;

        assert_eq!(files.len(), 1);

        if let Some(file) = files.first() {
            assert_eq!(
                file.path.to_string_lossy().to_string(),
                "/test/tuned/tuned-main.conf".to_string()
            );

            let expected_content = r#"# Managed by spec
daemon = 1
default_instance_priority = 0
dynamic_tuning = 1
reapply_sysctl = 1
recommend_command = 0
sleep_interval = 1
update_interval = 10"#;

            assert_eq!(expected_content, file.content);
        }

        Ok(())
    }
}
