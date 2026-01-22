use std::path::PathBuf;

use anyhow::Result;

use crate::types::{
    managed_service::{ManagedService, Plan},
    paths::Paths,
};

use super::SshService;

pub struct SshSystemService;

impl ManagedService for SshSystemService {
    fn name(&self) -> &str {
        "ssh_system"
    }

    fn plan(&self, config: &toml::Table, paths: &Paths) -> Result<Plan> {
        super::plan(self, config, paths)
    }
}

impl SshService for SshSystemService {
    fn config_path(paths: &Paths) -> PathBuf {
        paths.system_config.join("ssh/ssh_config")
    }

    fn requires_root() -> bool {
        true
    }
}
