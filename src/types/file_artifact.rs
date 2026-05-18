use anyhow::{Result, bail};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Debug, PartialEq)]
pub struct FileArtifact {
    pub path: PathBuf,
    pub content: String,
    pub permissions: u32,
    pub requires_root: bool,
}

impl FileArtifact {
    pub fn current_content(&self) -> Result<Option<Vec<u8>>> {
        if self.requires_root {
            let output = Command::new("sudo")
                .arg("cat")
                .arg(&self.path)
                .stderr(Stdio::null())
                .output()?;

            if output.status.success() {
                Ok(Some(output.stdout))
            } else {
                Ok(None)
            }
        } else if self.path.exists() {
            Ok(Some(std::fs::read(&self.path)?))
        } else {
            Ok(None)
        }
    }

    pub fn current_mode(&self) -> Result<u32> {
        use std::os::unix::fs::PermissionsExt;

        if self.requires_root {
            let output = Command::new("sudo")
                .arg("stat")
                .arg("-c")
                .arg("%a")
                .arg(&self.path)
                .output()?;

            if !output.status.success() {
                bail!("Failed to stat {:?}", self.path);
            }

            let s = std::str::from_utf8(&output.stdout)?;
            Ok(u32::from_str_radix(s.trim(), 8)?)
        } else {
            Ok(std::fs::metadata(&self.path)?.permissions().mode() & 0o777)
        }
    }

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

        println!(
            "Installing {} to {}",
            file.path().display(),
            &self.path.display()
        );

        if !status.success() {
            bail!("Failed to install file to {:?}", self.path);
        }

        Ok(())
    }
}
