use anyhow::{Result, bail};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, PartialEq)]
pub struct FileArtifact {
    pub path: PathBuf,
    pub content: String,
    pub permissions: u32,
    pub requires_root: bool,
}

impl FileArtifact {
    pub fn write(&self) -> Result<()> {
        let mut file = tempfile::NamedTempFile::new()?;
        file.write_all(self.content.as_bytes())?;

        let mut cmd = Command::new("sudo");
        cmd.arg("install").arg("-D");

        if self.requires_root {
            cmd.arg("-o").arg("root").arg("-g").arg("root");
        };

        let status = cmd
            .arg("-m")
            .arg(format!("{:o}", self.permissions))
            .arg(file.path())
            .arg(&self.path)
            .status()?;

        if !status.success() {
            bail!("Failed to install file to {:?}", self.path);
        }

        Ok(())
    }
}
