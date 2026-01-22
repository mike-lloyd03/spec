use std::{fs, path::PathBuf, str::FromStr};

use anyhow::Result;
use directories::{BaseDirs, ProjectDirs};

use crate::utils::dir_from_env_or_default;

#[derive(Debug, Default)]
pub struct Paths {
    pub spec_config: PathBuf,
    pub spec_state: PathBuf,
    pub user_home: PathBuf,
    pub user_config: PathBuf,
    pub system_config: PathBuf,
}

impl Paths {
    pub fn load() -> Result<Self> {
        let base_dirs = BaseDirs::new().expect("");

        let project_dirs =
            ProjectDirs::from("", "", "spec").expect("project directories should create");

        let spec_config =
            dir_from_env_or_default("SPEC_CONFIG_DIR", project_dirs.config_dir().to_owned())?;

        fs::create_dir_all(&spec_config)?;

        let spec_state = dir_from_env_or_default(
            "SPEC_STATE_DIR",
            project_dirs
                .state_dir()
                .expect("project state directory should create")
                .to_owned(),
        )?;
        fs::create_dir_all(&spec_state)?;

        let user_home =
            dir_from_env_or_default("SPEC_USER_HOME_DIR", base_dirs.home_dir().to_owned())?;

        let user_config =
            dir_from_env_or_default("SPEC_USER_CONFIG_DIR", base_dirs.config_dir().to_owned())?;

        let system_config =
            dir_from_env_or_default("SPEC_SYSTEM_CONFIG_DIR", PathBuf::from_str("/etc")?)?;

        Ok(Self {
            spec_config,
            spec_state,
            user_home,
            user_config,
            system_config,
        })
    }
}
