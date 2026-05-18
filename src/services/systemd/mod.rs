use std::path::PathBuf;

use anyhow::Result;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{
    adapters::key_value::KeyValueAdapter,
    types::{
        file_artifact::FileArtifact,
        managed_service::{ManagedService, Plan, ServiceState},
        paths::Paths,
    },
};

mod systemd_system;
mod systemd_user;
pub use systemd_system::SystemdSystemService;
pub use systemd_user::SystemdUserService;

#[derive(Deserialize)]
struct SystemdConfig {
    pub services: Option<Vec<SystemdUnit>>,
    pub timers: Option<Vec<SystemdTimer>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all(serialize = "PascalCase"))]
struct SystemdUnit {
    pub name: String,
    pub unit: Unit,
    pub service: Service,
    pub install: Option<Install>,

    #[serde(skip_serializing)]
    pub enabled: Option<bool>,
    #[serde(skip_serializing)]
    pub running: Option<bool>,
}

#[derive(Deserialize)]
struct SystemdTimer {
    pub name: String,
    pub unit: Unit,
    pub timer: Timer,
    pub install: Option<Install>,

    #[serde(skip_serializing)]
    pub enabled: Option<bool>,
    #[serde(skip_serializing)]
    pub running: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct Unit {
    pub description: String,
    pub after: Option<Vec<String>>,
    pub requires: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct Service {
    pub exec_start: String,
    pub exec_start_pre: Option<Vec<String>>,
    pub exec_start_post: Option<Vec<String>>,
    pub r#type: Option<ServiceType>,
    pub restart: Option<String>,
    pub user: Option<String>,
    pub environment: Option<IndexMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "kebab-case"))]
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct Install {
    pub wanted_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct Timer {
    pub accuracy_sec: Option<String>,
    pub defer_reactivation: Option<bool>,
    pub fixed_random_delay: Option<bool>,
    pub on_active_sec: Option<String>,
    pub on_boot_sec: Option<String>,
    pub on_calendar: Option<Vec<String>>,
    pub on_clock_change: Option<String>,
    pub on_startup_sec: Option<String>,
    pub on_timezone_change: Option<String>,
    pub on_unit_active_sec: Option<String>,
    pub on_unit_inactive_sec: Option<String>,
    pub persistent: Option<bool>,
    pub randomized_delay_sec: Option<String>,
    pub randomized_offset_sec: Option<String>,
    pub remain_after_elapse: Option<bool>,
    pub unit: Option<String>,
    pub wake_system: Option<bool>,
}

impl SystemdUnit {
    pub fn render(&self) -> Result<String> {
        let mut adapter = KeyValueAdapter::new("=", "#");

        adapter.comment("Managed by spec");

        adapter
            .push_line("[Unit]")
            .parse_struct(&self.unit)?
            .empty_line();

        adapter
            .push_line("[Service]")
            .parse_struct(&self.service)?
            .empty_line();

        if let Some(install) = &self.install {
            adapter.push_line("[Install]").parse_struct(install)?;
        }

        Ok(adapter.build())
    }
}

impl SystemdTimer {
    pub fn render(&self) -> Result<String> {
        let mut adapter = KeyValueAdapter::new("=", "#");

        adapter.comment("Managed by spec");

        adapter
            .push_line("[Unit]")
            .parse_struct(&self.unit)?
            .empty_line();

        adapter
            .push_line("[Timer]")
            .parse_struct(&self.timer)?
            .empty_line();

        if let Some(install) = &self.install {
            adapter.push_line("[Install]").parse_struct(install)?;
        }

        Ok(adapter.build())
    }
}

pub trait SystemdService {
    fn config_path(paths: &Paths) -> PathBuf;

    fn requires_root() -> bool;
}

fn plan<S: ManagedService + SystemdService>(
    service: &S,
    config_table: &toml::Table,
    paths: &Paths,
) -> Result<Plan> {
    let config: SystemdConfig = service.parse_config(config_table)?;

    let mut files = vec![];
    let mut service_states = vec![];

    if let Some(units) = config.services {
        for unit in units {
            let service_name = format!("{}.service", unit.name);

            let (file, state) = create_files_and_states::<S>(
                paths,
                &service_name,
                &unit.render()?,
                unit.running,
                unit.enabled,
            )?;

            files.push(file);
            service_states.push(state);
        }
    }

    if let Some(timers) = config.timers {
        for timer in timers {
            let service_name = format!("{}.timer", timer.name);

            let (file, state) = create_files_and_states::<S>(
                paths,
                &service_name,
                &timer.render()?,
                timer.running,
                timer.enabled,
            )?;

            files.push(file);
            service_states.push(state);
        }
    }

    Ok(Plan {
        files,
        addl_service_states: Some(service_states),
        ..Default::default()
    })
}

fn create_files_and_states<S: ManagedService + SystemdService>(
    paths: &Paths,
    service_name: &str,
    file_content: &str,
    running: Option<bool>,
    enabled: Option<bool>,
) -> Result<(FileArtifact, ServiceState)> {
    let file = FileArtifact {
        path: S::config_path(paths).join(service_name),
        content: file_content.to_string(),
        permissions: 0o644,
        requires_root: S::requires_root(),
    };

    let state = if S::requires_root() {
        ServiceState::new(service_name, enabled, running).requires_root(true)
    } else {
        ServiceState::builder(service_name)
            .requires_root(false)
            .running(running)
            .enabled(enabled)
            .reload_cmd(&format!("systemctl --user reload {}", service_name))
            .start_cmd(&format!("systemctl --user start {}", service_name))
            .enable_cmd(&format!("systemctl --user enable {}", service_name))
            .stop_cmd(&format!("systemctl --user stop {}", service_name))
            .disable_cmd(&format!("systemctl --user disable {}", service_name))
    };
    Ok((file, state))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use toml::Table;

    use super::*;

    #[test]
    fn test_plan() -> Result<()> {
        let service = SystemdSystemService;
        let paths = Paths::default();

        let config_table = Table::from_str(
            r#"
[[services]]
name = "test1"
running = true
enabled = true
unit = { description = "test1 desc"}
service = {exec_start = "test_program"}

[[services]]
name = "test2"

[services.unit]
description = "test2 desc"

[services.service]
exec_start = "test_program_2"
environment = { VAR1 = "val1", VAR2 = "val2"}

[[timers]]
name = "test1"
running = true
enabled = true

[timers.unit]
description = "Timer for test1"

[timers.timer]
on_calendar = ["hourly"]
unit = "test1"
wake_system = true

[[timers]]
name = "test2"

[timers.unit]
description = "Timer for test2"

[timers.timer]
on_calendar = ["Mon..Fri 22:30", "Sat,Sun 20:00"]
unit = "test2"
        "#,
        )?;

        let expected_files = [
            FileArtifact {
                path: SystemdSystemService::config_path(&paths).join("test1.service"),
                content: r#"
# Managed by spec
[Unit]
Description=test1 desc

[Service]
ExecStart=test_program
"#
                .to_string(),
                permissions: 0o644,
                requires_root: true,
            },
            FileArtifact {
                path: SystemdSystemService::config_path(&paths).join("test2.service"),
                content: r#"
# Managed by spec
[Unit]
Description=test2 desc

[Service]
Environment='VAR1=val1' 'VAR2=val2'
ExecStart=test_program_2
"#
                .to_string(),
                permissions: 0o644,
                requires_root: true,
            },
            FileArtifact {
                path: SystemdSystemService::config_path(&paths).join("test1.timer"),
                content: r#"
# Managed by spec
[Unit]
Description=Timer for test1

[Timer]
OnCalendar=hourly
Unit=test1
WakeSystem=true
"#
                .to_string(),
                permissions: 0o644,
                requires_root: true,
            },
            FileArtifact {
                path: SystemdSystemService::config_path(&paths).join("test2.timer"),
                content: r#"
# Managed by spec
[Unit]
Description=Timer for test2

[Timer]
OnCalendar=Mon..Fri 22:30
OnCalendar=Sat,Sun 20:00
Unit=test2
"#
                .to_string(),
                permissions: 0o644,
                requires_root: true,
            },
        ];

        let plan = service.plan(&config_table, &paths)?;
        let files = plan.files;
        let service_states = plan
            .addl_service_states
            .expect("addl_service_states should be Some");

        assert_eq!(4, files.len());
        assert_eq!(4, service_states.len());

        for (i, file) in files.iter().enumerate() {
            assert_eq!(file.path, expected_files[i].path);

            let expected = expected_files[i].content.trim();
            let actual = file.content.trim();

            assert_eq!(expected, actual, "For file {}", file.path.to_string_lossy());
        }

        let expected_states = [
            ServiceState::new("test1.service", Some(true), Some(true)),
            ServiceState::new("test2.service", None, None),
            ServiceState::new("test1.timer", Some(true), Some(true)),
            ServiceState::new("test2.timer", None, None),
        ];

        for (i, state) in service_states.iter().enumerate() {
            assert_eq!(expected_states[i].name, state.name);
            assert_eq!(expected_states[i].running, state.running);
            assert_eq!(expected_states[i].enabled, state.enabled);
        }

        Ok(())
    }
}
