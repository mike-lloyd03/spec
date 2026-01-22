use std::path::PathBuf;

use anyhow::Result;

use crate::{
    services::systemd::SystemdService,
    types::{
        managed_service::{ManagedService, Plan},
        paths::Paths,
    },
};

pub struct SystemdSystemService;

impl ManagedService for SystemdSystemService {
    fn name(&self) -> &str {
        "systemd_system"
    }

    fn plan(&self, config_table: &toml::Table, paths: &Paths) -> Result<Plan> {
        super::plan(self, config_table, paths)
    }
}

impl SystemdService for SystemdSystemService {
    fn config_path(paths: &Paths) -> PathBuf {
        paths.system_config.join("systemd/system")
    }

    fn requires_root() -> bool {
        true
    }
}
