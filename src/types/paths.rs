use std::{fs, path::PathBuf, str::FromStr};

use anyhow::Result;

use crate::utils::dir_from_env_or_default;

#[derive(Debug, Default)]
pub struct Paths {
    pub spec_config: PathBuf,
    pub user_state: PathBuf,
    pub user_config: PathBuf,
    pub system_config: PathBuf,
}

impl Paths {
    pub fn load() -> Result<Self> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("spec");

        let spec_config =
            dir_from_env_or_default("SPEC_CONFIG_DIR", xdg_dirs.config_home.clone().unwrap())?;

        let user_state = dir_from_env_or_default("SPEC_STATE_DIR", xdg_dirs.state_home.unwrap())?;
        fs::create_dir_all(&user_state)?;

        let system_config =
            dir_from_env_or_default("SPEC_SYSTEM_CONFIG_DIR", PathBuf::from_str("/env")?)?;

        let user_config =
            dir_from_env_or_default("SPEC_USER_CONFIG_DIR", xdg_dirs.config_home.unwrap())?;

        Ok(Self {
            spec_config,
            user_state,
            user_config,
            system_config,
        })
    }
}
