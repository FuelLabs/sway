use fuel_crypto::Hasher;

/// Hashes a given data to Sha256
pub fn hash<T: Into<Vec<u8>>>(data: T) -> anyhow::Result<Vec<u8>> {
    let mut hasher = Hasher::default();
    hasher.input(data.into());
    Ok(hasher.finalize().to_vec())
}
