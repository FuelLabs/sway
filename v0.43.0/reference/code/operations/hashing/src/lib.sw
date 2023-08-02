library;

// ANCHOR: import_sha256
use std::hash::sha256;
// ANCHOR_END: import_sha256
// ANCHOR: import_keccak256
use std::hash::keccak256;
// ANCHOR_END: import_keccak256
// ANCHOR: sha256
fn sha256_hashing(age: u64, name: str[5], status: bool) -> b256 {
    sha256((age, name, status))
}
// ANCHOR_END: sha256
// ANCHOR: keccak256
fn keccak256_hashing(age: u64, name: str[5], status: bool) -> b256 {
    keccak256((age, name, status))
}
// ANCHOR_END: keccak256
