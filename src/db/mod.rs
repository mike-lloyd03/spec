use std::path::Path;

use color_eyre::eyre::Result;
use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};

pub mod types;

const MIGRATIONS: &[M<'_>] = &[M::up(
    r#"
    CREATE TABLE runs (
        id INTEGER PRIMARY KEY,
        data JSON NOT NULL,
        sys_config_dir TEXT NOT NULL
    );
    "#,
)];

pub fn connect_db(state_dir: &Path) -> Result<Connection> {
    let db_path = state_dir.join("data.db");

    let mut conn = Connection::open(db_path)?;

    let m = Migrations::from_slice(MIGRATIONS);

    m.to_latest(&mut conn)?;

    Ok(conn)
}
