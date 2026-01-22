use anyhow::Result;
use serde::{Deserialize, Serialize};
use toml::Table;

use crate::{
    adapters::key_value::{BoolStyle, KeyValueAdapter},
    types::{
        file_artifact::FileArtifact,
        managed_service::{ManagedService, Plan, ServiceConfig, ServiceState},
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

    fn plan(&self, config: &Table, paths: &Paths) -> Result<Plan> {
        let config: TuneDConfig = self.parse_config(config)?;
        let mut files = Vec::new();

        let mut adapter = KeyValueAdapter::new(" = ", "#").bool_style(BoolStyle::OneZero);

        adapter.comment("Managed by spec");

        if adapter.parse_struct(&config).is_ok() {
            files.push(FileArtifact {
                path: paths.system_config.join("tuned/tuned-main.conf"),
                content: adapter.build(),
                permissions: 0o644,
                requires_root: true,
            });
        }

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

        let plan = service.plan(&config_table, &paths)?;

        assert_eq!(plan.files.len(), 1);

        if let Some(file) = plan.files.first() {
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
