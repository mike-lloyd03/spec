use anyhow::Result;

use crate::types::{
    managed_service::{FileArtifact, ManagedService, ServiceState},
    paths::Paths,
};

pub struct SshSystemService;

impl ManagedService for SshSystemService {
    fn name(&self) -> &str {
        "ssh_system"
    }

    fn plan(
        &self,
        config: &toml::Table,
        paths: &Paths,
    ) -> Result<(Vec<FileArtifact>, Option<ServiceState>)> {
        super::plan(self, config, paths)
    }
}
