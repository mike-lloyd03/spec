use crate::adapters::key_value::KeyValueAdapter;
use crate::types::file_artifact::FileArtifact;
use crate::types::managed_service::{ManagedService, Plan, ServiceConfig, ServiceState};
use crate::types::paths::Paths;

use anyhow::Result;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use toml::Table;

pub struct UdevService;

#[derive(Serialize, Deserialize, Default)]
struct UdevConfig {
    udev_log: Option<LogLevel>,
    children_max: Option<u16>,
    exec_delay: Option<u16>,
    event_timeout: Option<u16>,
    resolve_names: Option<ResolveNames>,
    timeout_signal: Option<String>,

    #[serde(skip_serializing)]
    rules: Option<IndexMap<String, String>>,

    #[serde(skip_serializing)]
    pub service: Option<ServiceConfig>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    #[default]
    Err,
    Info,
    Debug,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum ResolveNames {
    #[default]
    Early,
    Late,
    Never,
}

impl ManagedService for UdevService {
    fn name(&self) -> &str {
        "udev"
    }

    fn plan(&self, config: &Table, paths: &Paths) -> Result<Plan> {
        let config: UdevConfig = self.parse_config(config)?;

        let mut files = vec![];

        let mut adapter = KeyValueAdapter::new("=", "#");

        adapter.comment("Managed by spec");

        if adapter.parse_struct(&config).is_ok() {
            files.push(FileArtifact {
                path: paths.system_config.join("udev/udev.conf"),
                content: adapter.build(),
                permissions: 0o644,
                requires_root: true,
            });
        }

        if let Some(rules) = config.rules {
            for (filename, content) in rules {
                let prefixed_content = "# Managed by spec\n".to_string() + &content;

                files.push(FileArtifact {
                    path: paths
                        .system_config
                        .join(format!("udev/rules.d/{}", filename)),
                    content: prefixed_content,
                    permissions: 0o644,
                    requires_root: true,
                });
            }
        };

        let service_state = config.service.map(|s| {
            ServiceState::new("systemd-udevd", s.enabled, s.running)
                .reload_cmd("udevadm control --reload")
        });

        Ok(Plan {
            files,
            service_state,
            ..Default::default()
        })
    }
}
