use std::{fs, path::PathBuf, str::FromStr};

use color_eyre::Result;
use toml::Table;

pub struct Config {
    pub base_path: PathBuf,
}

impl Config {
    pub fn load(base_path_str: Option<String>) -> Result<Self> {
        let base_path = if let Some(config_dir_str) = base_path_str {
            PathBuf::from_str(&config_dir_str)?
        } else {
            xdg::BaseDirectories::new()
                .get_config_home()
                .expect("User HOME should exist")
                .join("spec")
        };

        Ok(Self { base_path })
    }

    pub fn load_services(&self) -> Result<Table> {
        let mut merged = Table::new();

        let services_dir = self.base_path.join("services");
        if services_dir.exists() {
            for entry in fs::read_dir(services_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    let content = fs::read_to_string(&path)?;
                    let table: Table = toml::from_str(&content)?;

                    Self::merge_tables(&mut merged, table);
                }
            }
        }

        Ok(merged)
    }

    fn merge_tables(base: &mut Table, incoming: Table) {
        for (key, value) in incoming {
            base.insert(key, value);
        }
    }
}
