use fuel_crypto::Hasher;
use serde_json::json;

/// Hashes a given data to Sha256
pub fn hash<T: Into<Vec<u8>>>(data: T) -> anyhow::Result<serde_json::Value> {
    let mut hasher = Hasher::default();
    hasher.input(data.into());
    Ok(json!(hex::encode(hasher.finalize())))
}
