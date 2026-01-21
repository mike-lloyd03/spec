use crate::adapters::key_value::KeyValueAdapter;
use crate::types::managed_service::{FileArtifact, ManagedService, ServiceConfig, ServiceState};
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

    fn plan(
        &self,
        config: &Table,
        paths: &Paths,
    ) -> Result<(Vec<FileArtifact>, Option<ServiceState>)> {
        let config: UdevConfig = self.parse_config(config)?;

        let mut files = vec![];

        let mut adapter = KeyValueAdapter::new("=", "#");

        adapter.comment("Managed by spec");

        if adapter.parse_struct(&config).is_ok() {
            files.push(FileArtifact {
                path: paths.system_config.join("udev/udev.conf"),
                content: adapter.build(),
                permissions: 0o644,
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
                });
            }
        };

        let service_state = config.service.map(|s| {
            ServiceState::new("systemd-udevd", s.enabled, s.running)
                .reload_cmd("udevadm control --reload")
        });

        // let service_state = config.service.map(|state| ServiceState {
        //     name: self.name().to_string(),
        //     enabled: state.enabled,
        //     running: state.running,
        //     reload_cmd: Some("udevadm control --reload".to_string()),
        // });

        Ok((files, service_state))
    }
}
