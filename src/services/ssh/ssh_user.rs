use anyhow::Result;

use crate::types::{
    managed_service::{FileArtifact, ManagedService, ServiceState},
    paths::Paths,
};

pub struct SshUserService;

impl ManagedService for SshUserService {
    fn name(&self) -> &str {
        "ssh_user"
    }

    fn plan(
        &self,
        config: &toml::Table,
        paths: &Paths,
    ) -> Result<(Vec<FileArtifact>, Option<ServiceState>)> {
        super::plan(self, config, paths)
    }
}
