use std::path::PathBuf;

use anyhow::Result;

use crate::types::{
    managed_service::{ManagedService, Plan},
    paths::Paths,
};

use super::SystemdService;

pub struct SystemdUserService;

impl ManagedService for SystemdUserService {
    fn name(&self) -> &str {
        "systemd_user"
    }

    fn plan(&self, config_table: &toml::Table, paths: &Paths) -> Result<Plan> {
        super::plan(self, config_table, paths)
    }
}

impl SystemdService for SystemdUserService {
    fn config_path(paths: &Paths) -> PathBuf {
        paths.user_config.join("systemd/user")
    }

    fn requires_root() -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use toml::Table;

    use crate::types::{file_artifact::FileArtifact, managed_service::ServiceState};

    use super::*;

    #[test]
    fn test_plan() -> Result<()> {
        let service = SystemdUserService;
        let paths = Paths::default();

        let config_table = Table::from_str(
            r#"
[[services]]
name = "test1"
running = true
enabled = true
unit = { description = "test1 desc"}
service = {exec_start = "test_program"}
"#,
        )?;

        let expected_file = FileArtifact {
            path: PathBuf::from_str("systemd/user/test1.service")?,
            content: r#"# Managed by spec
[Unit]
Description=test1 desc

[Service]
ExecStart=test_program
"#
            .to_string(),
            permissions: 0o644,
            requires_root: false,
        };

        let expected_state = ServiceState::builder("test1.service")
            .requires_root(false)
            .enabled(Some(true))
            .running(Some(true))
            .reload_cmd("systemctl --user reload test1.service")
            .start_cmd("systemctl --user start test1.service")
            .enable_cmd("systemctl --user enable test1.service")
            .stop_cmd("systemctl --user stop test1.service")
            .disable_cmd("systemctl --user disable test1.service");

        let plan = service.plan(&config_table, &paths)?;
        let file = plan.files.first().unwrap();
        let state = plan.addl_service_states.unwrap().first().unwrap().clone();

        assert_eq!(&expected_file, file);
        assert_eq!(expected_state, state);

        Ok(())
    }
}
