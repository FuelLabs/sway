library;

// ANCHOR: import
use std::hash::*;
// ANCHOR_END: import
// ANCHOR: sha256
fn sha256_hashing(age: u64, name: str, status: bool) -> b256 {
    let mut hasher = Hasher::new();
    age.hash(hasher);
    hasher.write_str(name);
    status.hash(hasher);
    hasher.sha256()
}
// ANCHOR_END: sha256
// ANCHOR: keccak256
fn keccak256_hashing(age: u64, name: str, status: bool) -> b256 {
    let mut hasher = Hasher::new();
    age.hash(hasher);
    hasher.write_str(name);
    status.hash(hasher);
    hasher.keccak256()
}
// ANCHOR_END: keccak256
