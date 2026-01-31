use std::{env, path::PathBuf, str::FromStr};

use anyhow::Result;
use sha2::{Digest, Sha256};

pub fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

pub fn bytes_to_string(bytes: &[u8]) -> Result<String> {
    Ok(str::from_utf8(bytes)?.to_string())
}

pub fn dir_from_env_or_default(var_name: &str, default: PathBuf) -> Result<PathBuf> {
    Ok(if let Ok(dir) = env::var(var_name) {
        PathBuf::from_str(&dir)?
    } else {
        default
    })
}
