use serde::{Deserialize, Serialize};

use crate::types::{
    file_artifact::FileArtifact,
    managed_service::{ManagedService, Plan},
};

pub struct SudoersService;

#[derive(Serialize, Deserialize, Default)]
struct SudoersConfig {
    pub user_specs: Vec<UserSpec>,
}

#[derive(Serialize, Deserialize, Default)]
struct UserSpec {
    pub user: String,
    pub host: String,
    pub run_as: RunAsSpec,
    pub command: String,
}

#[derive(Serialize, Deserialize, Default)]
struct RunAsSpec {
    user: String,
    group: Option<String>,
}

impl ManagedService for SudoersService {
    fn name(&self) -> &str {
        "sudoers"
    }

    fn plan(
        &self,
        config: &toml::Table,
        paths: &crate::types::paths::Paths,
    ) -> anyhow::Result<crate::types::managed_service::Plan> {
        let config: SudoersConfig = self.parse_config(config)?;

        let path = paths.system_config.join("sudoers.d/99_spec");

        let mut content = String::from("# Managed by spec\n");

        for s in config.user_specs {
            let run_as = if let Some(group) = s.run_as.group {
                format!("{}:{}", s.run_as.user, group)
            } else {
                s.run_as.user
            };

            let spec_str = format!("{} {}=({}) {}\n", s.user, s.host, run_as, s.command);
            content.push_str(&spec_str);
        }

        let config_file = FileArtifact {
            path,
            content,
            permissions: 0o440,
            requires_root: true,
        };

        Ok(Plan {
            files: vec![config_file],
            service_state: None,
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::types::paths::Paths;

    use super::*;
    use anyhow::Result;
    use toml::Table;

    #[test]
    fn test_plan() -> Result<()> {
        let config_table = Table::from_str(
            r#"
[[user_specs]]
user = "test"
host = "ALL"
run_as = {
    user = "ALL",
    group = "ALL"
}
command = "ALL"

[[user_specs]]
user = "%sudo"
host = "ALL"
run_as = { user = "ALL", group = "ALL" }
command = "ALL"

[[user_specs]]
user = "bob"
host = "ALL"
run_as = { user = "root" }
command = "/usr/bin/apt, /usr/bin/systemctl"
"#,
        )?;

        let service = SudoersService;
        let paths = Paths {
            system_config: "/test".into(),
            ..Default::default()
        };

        let plan = service.plan(&config_table, &paths)?;

        assert_eq!(plan.files.len(), 1);

        if let Some(file) = plan.files.first() {
            assert_eq!(
                file.path.to_string_lossy().to_string(),
                "/test/sudoers.d/99_spec".to_string()
            );

            let expected_content = r#"# Managed by spec
test ALL=(ALL:ALL) ALL
%sudo ALL=(ALL:ALL) ALL
bob ALL=(root) /usr/bin/apt, /usr/bin/systemctl
"#;

            assert_eq!(expected_content.trim(), file.content.trim());
        }

        Ok(())
    }
}
