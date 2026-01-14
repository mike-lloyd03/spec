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
