use anyhow::Result;
use indexmap::IndexMap;
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

pub struct SddmService;

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all(serialize = "PascalCase"))]
struct SddmConfig {
    pub general: Option<General>,
    pub theme: Option<Theme>,
    pub x11: Option<X11>,
    pub wayland: Option<Wayland>,
    pub users: Option<Users>,
    pub autologin: Option<Autologin>,
    #[serde(skip_serializing)]
    as_dropin: Option<bool>,

    #[serde(skip_serializing)]
    dropins: Option<IndexMap<String, String>>,

    #[serde(skip_serializing)]
    pub service: Option<ServiceConfig>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum DisplayServer {
    #[default]
    X11,
    X11User,
    Wayland,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Numlock {
    On,
    Off,
    #[default]
    None,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all(serialize = "PascalCase"))]
struct General {
    pub display_server: Option<DisplayServer>,
    pub halt_command: Option<String>,
    pub reboot_command: Option<String>,
    pub numlock: Option<Numlock>,
    pub input_method: Option<String>,
    pub namespaces: Option<Vec<String>>,
    pub greeter_environment: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all(serialize = "PascalCase"))]
struct Theme {
    pub theme_dir: Option<String>,
    pub current: Option<String>,
    pub faces_dir: Option<String>,
    pub cursor_theme: Option<String>,
    pub cursor_size: Option<String>,
    pub font: Option<String>,
    pub enable_avatars: Option<bool>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all(serialize = "PascalCase"))]
struct X11 {
    pub server_path: Option<String>,
    pub server_arguments: Option<String>,
    pub xephyr_path: Option<String>,
    pub session_dir: Option<Vec<String>>,
    pub session_command: Option<String>,
    pub session_log_file: Option<String>,
    pub display_command: Option<String>,
    pub display_stop_command: Option<String>,
    pub enable_hi_dpi: Option<bool>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all(serialize = "PascalCase"))]
struct Wayland {
    pub compositor_command: Option<String>,
    pub session_dir: Option<Vec<String>>,
    pub session_command: Option<String>,
    pub session_log_file: Option<String>,
    pub enable_hi_dpi: Option<bool>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all(serialize = "PascalCase"))]
struct Users {
    pub default_path: Option<String>,
    pub minimum_uid: Option<u32>,
    pub maximum_uid: Option<u32>,
    pub hide_users: Option<Vec<String>>,
    pub hide_shells: Option<Vec<String>>,
    pub remember_last_user: Option<bool>,
    pub remember_last_session: Option<bool>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all(serialize = "PascalCase"))]
struct Autologin {
    pub user: Option<String>,
    pub session: Option<String>,
    pub relogin: Option<bool>,
}

impl ManagedService for SddmService {
    fn name(&self) -> &str {
        "sddm"
    }

    fn plan(&self, config: &Table, paths: &Paths) -> Result<Plan> {
        let config: SddmConfig = self.parse_config(config)?;

        let mut files = vec![];

        let mut adapter = IniAdapter::new();
        let content = adapter
            .push_line("# Managed by spec")
            .section("General", config.general)?
            .section("Theme", config.theme)?
            .section("X11", config.x11)?
            .section("Wayland", config.wayland)?
            .section("Users", config.users)?
            .section("Autologin", config.autologin)?
            .build();

        let path = if config.as_dropin.is_some_and(|b| b) {
            paths.system_config.join("sddm.conf.d/10-spec.conf")
        } else {
            paths.system_config.join("sddm.conf")
        };

        let config_file = FileArtifact {
            path,
            content,
            permissions: 0o644,
            requires_root: true,
        };

        files.push(config_file);

        if let Some(dropins) = config.dropins {
            for (filename, content) in dropins {
                let prefixed_content = "# Managed by spec\n".to_string() + &content;

                files.push(FileArtifact {
                    path: paths
                        .system_config
                        .join(format!("sddm.conf.d/{}", filename)),
                    content: prefixed_content,
                    permissions: 0o644,
                    requires_root: true,
                });
            }
        };

        let service_state = config
            .service
            .map(|s| ServiceState::new("sddm", s.enabled, s.running));

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
[general]
display_server = "wayland"
greeter_environment = "QT_WAYLAND_SHELL_INTEGRATION=layer-shell"

[wayland]
compositor_command = "kwin_wayland --drm --no-lockscreen --no-global-shortcuts --locale1"
session_dir = ["/usr/local/share/wayland-sessions", "/usr/share/wayland-sessions"]

[users]
minimum_uid = 1000
maximum_uid = 1100

[autologin]
user = "bob"
"#,
        )?;

        let service = SddmService;
        let paths = Paths {
            system_config: "/test".into(),
            ..Default::default()
        };

        let plan = service.plan(&config_table, &paths)?;

        assert_eq!(plan.files.len(), 1);

        if let Some(file) = plan.files.first() {
            assert_eq!(
                file.path.to_string_lossy().to_string(),
                "/test/sddm.conf".to_string()
            );

            let expected_content = r#"# Managed by spec
[General]
DisplayServer=wayland
GreeterEnvironment=QT_WAYLAND_SHELL_INTEGRATION=layer-shell

[Wayland]
CompositorCommand=kwin_wayland --drm --no-lockscreen --no-global-shortcuts --locale1
SessionDir=/usr/local/share/wayland-sessions,/usr/share/wayland-sessions

[Users]
MaximumUid=1100
MinimumUid=1000

[Autologin]
User=bob"#;

            assert_eq!(expected_content.trim(), file.content.trim());
        }

        Ok(())
    }
}
