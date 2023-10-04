use sha3::{Digest, Keccak256};

/// Hashes a given data using Keccak256
pub fn hash<T: AsRef<[u8]>>(data: T) -> anyhow::Result<Vec<u8>> {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    Ok(hasher.finalize().to_vec())
}
