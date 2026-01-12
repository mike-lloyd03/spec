use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use color_eyre::eyre::{Error, Result};
use rusqlite::{Connection, Error::InvalidColumnType, Row, types::Type::Text};
use toml::Table;

pub struct Run {
    pub id: u32,
    pub data: Table,
    pub sys_config_dir: PathBuf,
    pub managed_files: Vec<String>,
}

struct RunDB {
    pub id: u32,
    pub data: String,
    pub sys_config_dir: String,
    pub managed_files: String,
}

impl TryFrom<&Run> for RunDB {
    type Error = color_eyre::eyre::Error;

    fn try_from(value: &Run) -> Result<RunDB> {
        let data_value: toml::Value = value.data.clone().into();
        let data = serde_json::to_string(&data_value)?;

        let sys_config_dir = value
            .sys_config_dir
            .to_str()
            .ok_or_else(|| Error::msg("Failed to convert path to string"))?
            .to_string();

        let files_value: toml::Value = value.managed_files.clone().into();
        let managed_files = serde_json::to_string(&files_value)?;

        Ok(RunDB {
            id: value.id,
            data,
            sys_config_dir,
            managed_files,
        })
    }
}

impl Run {
    pub fn new(data: Table, sys_config_dir: &Path) -> Self {
        Self {
            id: 0,
            data,
            sys_config_dir: sys_config_dir.to_owned(),
            managed_files: vec![],
        }
    }

    pub fn create(&self, conn: &Connection) -> Result<()> {
        let run_db: RunDB = self.try_into()?;

        conn.execute(
            "INSERT INTO runs (data, sys_config_dir, managed_files) values (?1, ?2, ?3)",
            [run_db.data, run_db.sys_config_dir, run_db.managed_files],
        )?;
        Ok(())
    }

    pub fn get_previous(conn: &Connection) -> Result<Self> {
        let s: Self = conn.query_one(
            "SELECT id, data, sys_config_dir, managed_files from runs order by id desc limit 1",
            [],
            |row| row.try_into(),
        )?;
        Ok(s)
    }

    pub fn delete(&self, conn: &Connection) -> Result<()> {
        conn.execute("DELETE from runs where id = ?1", [self.id])?;
        Ok(())
    }

    pub fn update(&self, conn: &Connection) -> Result<()> {
        let run_db: RunDB = self.try_into()?;

        conn.execute(
            "UPDATE set (data = ?2, sys_config_dir = ?3, managed_files = ?4) ON runs WHERE id = ?1",
            [
                run_db.id.to_string(),
                run_db.data,
                run_db.sys_config_dir,
                run_db.managed_files,
            ],
        )?;
        Ok(())
    }
}

impl<'a> TryFrom<&Row<'a>> for Run {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        let data_str: String = row.get("data")?;
        let data: Table = serde_json::from_str(&data_str)
            .map_err(|e| InvalidColumnType(1, e.to_string(), Text))?;

        let sys_config_dir_str: String = row.get("sys_config_dir")?;
        let sys_config_dir = PathBuf::from_str(&sys_config_dir_str)
            .map_err(|e| InvalidColumnType(1, e.to_string(), Text))?;

        let files_str: String = row.get("managed_files")?;
        let files: Vec<String> = serde_json::from_str(&files_str)
            .map_err(|e| InvalidColumnType(1, e.to_string(), Text))?;

        Ok(Self {
            id: row.get("id")?,
            data,
            sys_config_dir,
            managed_files: files,
        })
    }
}
