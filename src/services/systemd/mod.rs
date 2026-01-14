use std::{fmt::Display, path::Path};

use indexmap::IndexMap;
use serde::Deserialize;

use crate::services::{FileArtifact, ManagedService};

pub struct SystemdService;

#[derive(Deserialize)]
struct SystemdConfig {
    pub unit_files: Option<Vec<SystemdUnit>>,
}

#[derive(Deserialize)]
struct SystemdUnit {
    pub name: String,
    pub unit: Unit,
    pub service: Service,
    pub install: Option<Install>,
}

#[derive(Debug, Deserialize)]
pub struct Unit {
    pub description: String,
    pub after: Option<Vec<String>>,
    pub requires: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct Service {
    pub exec_start: String,
    pub exec_start_pre: Option<Vec<String>>,
    pub exec_start_post: Option<Vec<String>>,
    pub r#type: Option<ServiceType>,
    pub restart: Option<String>,
    pub user: Option<String>,
    pub environment: Option<IndexMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub enum ServiceType {
    Simple,
    Exec,
    Forking,
    Oneshot,
    Dbus,
    Notify,
    NotifyReload,
    Idle,
}

impl Display for ServiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ServiceType::Simple => "simple",
            ServiceType::Exec => "exec",
            ServiceType::Forking => "forking",
            ServiceType::Oneshot => "oneshot",
            ServiceType::Dbus => "dbus",
            ServiceType::Notify => "notify",
            ServiceType::NotifyReload => "notify-reload",
            ServiceType::Idle => "idle",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Deserialize)]
pub struct Install {
    pub wanted_by: Vec<String>,
}

impl SystemdUnit {
    pub fn render(&self) -> String {
        let mut out = String::new();

        // Unit
        out.push_str("[Unit]\n");
        Self::add_line(&mut out, "Description", &self.unit.description);

        let after = self.unit.after.clone().map(|a| a.join(" "));
        Self::add_line_opt(&mut out, "After", &after);
        Self::add_line_opt_vec(&mut out, "Requires", &self.unit.requires);

        // Service
        out.push_str("\n[Service]\n");
        Self::add_line(&mut out, "ExecStart", &self.service.exec_start);
        Self::add_line_opt_vec(&mut out, "ExecStartPre", &self.service.exec_start_pre);
        Self::add_line_opt_vec(&mut out, "ExecStartPost", &self.service.exec_start_post);
        Self::add_line_opt(&mut out, "Type", &self.service.r#type);
        Self::add_line_opt(&mut out, "Restart", &self.service.restart);
        Self::add_line_opt(&mut out, "User", &self.service.user);
        let env: &Option<String> = &self
            .service
            .environment
            .as_ref()
            .map(|e| e.iter().map(|(k, v)| format!("'{k}={v}'")).collect());
        Self::add_line_opt(&mut out, "Environment", env);

        // Install
        if let Some(install) = &self.install {
            out.push_str("\n[Install]\n");
            let wanted_by = install.wanted_by.join(" ");
            Self::add_line(&mut out, "WantedBy", &wanted_by);
        }

        out
    }

    fn add_line(s: &mut String, key: &str, value: impl Display) {
        Self::add_line_opt(s, key, &Some(value.to_string()));
    }

    fn add_line_opt(s: &mut String, key: &str, value: &Option<impl Display>) {
        if let Some(v) = value {
            s.push_str(&format!("{key}={v}\n"));
        }
    }

    fn add_line_opt_vec(s: &mut String, key: &str, opt: &Option<Vec<impl Display>>) {
        if let Some(vals) = opt {
            for v in vals {
                s.push_str(&format!("{key}={v}\n"));
            }
        }
    }
}

impl ManagedService for SystemdService {
    fn name(&self) -> &str {
        "systemd"
    }

    fn plan(
        &self,
        config_table: &toml::Table,
        sys_config_dir: &Path,
    ) -> anyhow::Result<(Vec<super::FileArtifact>, Option<super::ServiceState>)> {
        let config: SystemdConfig = self.parse_config(config_table)?;

        let mut artifacts = vec![];

        if let Some(units) = config.unit_files {
            for unit in units {
                artifacts.push(FileArtifact {
                    path: sys_config_dir
                        .join("systemd/system/multi-user.target.wants")
                        .join(format!("{}.service", unit.name)),
                    content: unit.render(),
                    permissions: 0o644,
                });
            }
        }

        Ok((artifacts, None))
    }
}
