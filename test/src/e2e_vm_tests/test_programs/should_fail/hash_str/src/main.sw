script;

use std::hash::*;

fn sha256_str<T>(s: T) -> b256 {
    let mut hasher = Hasher::new();
    hasher.write_str(s);
    hasher.sha256()
}

fn main() -> u64 {

    let _hash = sha256_str(0_u8);

    5
}
