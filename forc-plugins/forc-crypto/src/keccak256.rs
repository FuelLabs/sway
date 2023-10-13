use sha3::{Digest, Keccak256};

/// Hashes a given data using Keccak256
pub fn hash<T: Into<Vec<u8>>>(data: T) -> anyhow::Result<Vec<u8>> {
    let mut hasher = Keccak256::new();
    hasher.update(data.into());
    Ok(hasher.finalize().to_vec())
}
