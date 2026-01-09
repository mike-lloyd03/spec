use std::collections::HashMap;
use std::path::PathBuf;

use crate::adapters::key_value::KeyValueAdapter;
use crate::services::{FileArtifact, ManagedService, ServiceState};

use color_eyre::Result;
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
    rules: HashMap<String, String>,
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
        sys_config_dir: &str,
    ) -> Result<(Vec<FileArtifact>, Option<ServiceState>)> {
        let config: UdevConfig = self.parse_config(config)?;

        let mut files = vec![];

        let mut adapter = KeyValueAdapter::new("=", "#");

        adapter.comment("Managed by spec");

        adapter.parse_struct(&config)?;

        files.push(FileArtifact {
            path: PathBuf::from(sys_config_dir).join("udev/udev.conf"),
            content: adapter.build(),
            permissions: 0o644,
        });

        for (filename, content) in config.rules {
            let prefixed_content = "# Managed by spec\n".to_string() + &content;
            files.push(FileArtifact {
                path: PathBuf::from(sys_config_dir).join(format!("udev/rules.d/{}", filename)),
                content: prefixed_content,
                permissions: 0o644,
            });
        }

        Ok((files, None))
    }
}
