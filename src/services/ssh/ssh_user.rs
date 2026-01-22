use std::path::PathBuf;

use anyhow::Result;

use crate::types::{
    managed_service::{ManagedService, Plan},
    paths::Paths,
};

use super::SshService;

pub struct SshUserService;

impl ManagedService for SshUserService {
    fn name(&self) -> &str {
        "ssh_user"
    }

    fn plan(&self, config: &toml::Table, paths: &Paths) -> Result<Plan> {
        super::plan(self, config, paths)
    }
}

impl SshService for SshUserService {
    fn config_path(paths: &Paths) -> PathBuf {
        paths.user_home.join(".ssh/config")
    }

    fn requires_root() -> bool {
        false
    }
}
