use color_eyre::eyre::Result;
use rusqlite::{Connection, Row};
use toml::Table;

pub struct Run {
    pub id: u32,
    pub data: Table,
    pub sys_config_dir: String,
}

impl Run {
    pub fn new(data: Table, sys_config_dir: String) -> Self {
        Self {
            id: 0,
            data,
            sys_config_dir,
        }
    }

    pub fn create(&self, conn: &Connection) -> Result<()> {
        let toml_value: toml::Value = self.data.clone().into();
        let json_data = serde_json::to_string(&toml_value)?;
        conn.execute(
            "INSERT INTO runs (data, sys_config_dir) values (?1, ?2)",
            [json_data, self.sys_config_dir.clone()],
        )?;
        Ok(())
    }

    pub fn get_previous(conn: &Connection) -> Result<Self> {
        let s: Self = conn.query_one(
            "SELECT id, data, sys_config_dir from runs order by id desc limit 1",
            [],
            |row| row.try_into(),
        )?;
        Ok(s)
    }
}

impl<'a> TryFrom<&Row<'a>> for Run {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        let data_str: String = row.get("data")?;
        let data: Table = serde_json::from_str(&data_str).map_err(|e| {
            rusqlite::Error::InvalidColumnType(1, e.to_string(), rusqlite::types::Type::Text)
        })?;

        Ok(Self {
            id: row.get("id")?,
            data,
            sys_config_dir: row.get("sys_config_dir")?,
        })
    }
}
