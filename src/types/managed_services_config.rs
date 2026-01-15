use std::{fs, path::Path};

use anyhow::Result;
use cliclack::log;
use serde::{Deserialize, Serialize};
use toml::{Table, Value};

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct ManagedServicesConfig {
    #[serde(default)]
    pub data: Table,
}

impl ManagedServicesConfig {
    pub fn load(config_dir: &Path) -> Result<Self> {
        let mut merged = Table::new();

        let services_dir = config_dir.join("services");

        if services_dir.exists() {
            for entry in fs::read_dir(services_dir)? {
                let path = entry?.path();

                if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    let content = fs::read_to_string(&path)?;
                    let table: Table = toml::from_str(&content)?;

                    merge_tables(&mut merged, table);
                }
            }
        } else {
            log::warning("Services configuration directory not found")?;
        }

        Ok(Self { data: merged })
    }
}

fn merge_tables(base: &mut Table, incoming: Table) {
    for (key, value) in incoming {
        match (base.get_mut(&key), value) {
            (Some(Value::Table(base_child)), Value::Table(incoming_child)) => {
                merge_tables(base_child, incoming_child);
            }
            (_, value) => {
                base.insert(key, value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use toml::Table;

    #[test]
    fn test_merge_tables() -> Result<()> {
        let mut base_table = Table::from_str(
            r#"
            [user]
            name = "bob"
            admin = false
            "#,
        )?;

        let incoming = Table::from_str(
            r#"
            [user]
            email = "bob@example.com"

            [system]
            arch = "x86_64"
            "#,
        )?;

        let expect = Table::from_str(
            r#"
            [user]
            name = "bob"
            admin = false
            email = "bob@example.com"

            [system]
            arch = "x86_64"
            "#,
        )?;

        merge_tables(&mut base_table, incoming);

        assert_eq!(expect, base_table);

        Ok(())
    }
}
