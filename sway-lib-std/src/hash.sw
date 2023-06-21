//! Utility functions for cryptographic hashing.
library;

use ::bytes::*;

pub struct Hasher {
    bytes: Bytes
}

impl Hasher {
    pub fn new() -> Self {
        Self { bytes: Bytes::new() }
    }

    /// Writes some data into this `Hasher`.
    pub fn write(ref mut self, bytes: Bytes) {
        self.bytes.append(bytes);
    }

    pub fn sha256(self) -> b256 {
        self.bytes.sha256()
    }

    pub fn keccak256(self) -> b256 {
        self.bytes.keccak256()
    }
}

impl Hasher {
    /// Writes a single `str` into this hasher.
    pub fn write_str<S>(ref mut self, s: S) {
        __check_str_type::<S>();
        let str_size = __size_of_str::<S>();
        let str_ptr = __addr_of(s);
        let mut bytes = Bytes::with_capacity(str_size);
        bytes.len = str_size;

        str_ptr.copy_bytes_to(bytes.buf.ptr(), str_size);
        self.write(bytes);
    }
}

pub trait Hash {
    fn hash(self, ref mut state: Hasher);
}

impl Hash for u8 {
    fn hash(self, ref mut state: Hasher) {
        let mut bytes = Bytes::with_capacity(1);
        bytes.push(self);
        state.write(bytes);
    }
}

impl Hash for u16 {
    fn hash(self, ref mut state: Hasher) {
        let mut bytes = Bytes::with_capacity(8); // one word capacity
        bytes.len = 2;

        asm(ptr: bytes.buf.ptr(), val: self, r1) {
            slli  r1 val i48;
            sw ptr r1 i0;
        };

        state.write(bytes);
    }
}

impl Hash for u32 {
    fn hash(self, ref mut state: Hasher) {
        let mut bytes = Bytes::with_capacity(8); // one word capacity
        bytes.len = 4;

        asm(ptr: bytes.buf.ptr(), val: self, r1) {
            slli  r1 val i32;
            sw ptr r1 i0;
        };

        state.write(bytes);
    }
}

impl Hash for u64 {
    fn hash(self, ref mut state: Hasher) {
        let mut bytes = Bytes::with_capacity(8); // one word capacity
        bytes.len = 8;

        asm(ptr: bytes.buf.ptr(), val: self) {
            sw ptr val i0;
        };

        state.write(bytes);
    }
}

impl Hash for b256 {
    fn hash(self, ref mut state: Hasher) {
        let mut bytes = Bytes::with_capacity(32); // four word capacity
        bytes.len = 32;

        let (word_1, word_2, word_3, word_4) = asm(r1: self) { r1: (u64, u64, u64, u64) };

        asm(ptr: bytes.buf.ptr(), val_1: word_1, val_2: word_2, val_3: word_3, val_4: word_4) {
            sw ptr val_1 i0;
            sw ptr val_2 i1;
            sw ptr val_3 i2;
            sw ptr val_4 i3;
        };

        state.write(bytes);
    }
}

impl Hash for bool {
    fn hash(self, ref mut state: Hasher) {
        let mut bytes = Bytes::with_capacity(1);
        if self {
            bytes.push(1_u8);
        } else {
            bytes.push(0_u8);
        }
        state.write(bytes);
    }
}

pub fn sha256<T>(s: T) -> b256 where T: Hash {
    let mut hasher = Hasher::new();
    s.hash(hasher);
    hasher.sha256()
}

pub fn keccak256<T>(s: T) -> b256 where T: Hash {
    let mut hasher = Hasher::new();
    s.hash(hasher);
    hasher.keccak256()
}

// Tests
//

#[test()]
fn test_hasher_sha256_str() {
    use ::assert::assert;
    let mut hasher = Hasher::new();
    hasher.write_str("test");
    let sha256 = hasher.sha256();
    assert(sha256 == 0x9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08);

    let mut hasher = Hasher::new();
    hasher.write_str("Fastest Modular Execution Layer!");
    let sha256 = hasher.sha256();
    assert(sha256 == 0x4a3cd7c8b44dbf7941e55179425f746adeaa97fe2d99b571fffee78e9b41743c);
}

// The hashes for the following test can be obtained in Rust by running the following script:
// https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=a2d83e9ea48b35a3e991c904c3451ed5
#[test()]
fn test_hasher_sha256_u8() {
    use ::assert::assert;
    let mut hasher = Hasher::new();
    0_u8.hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d);

    let mut hasher = Hasher::new();
    1_u8.hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x4bf5122f344554c53bde2ebb8cd2b7e3d1600ad631c385a5d7cce23c7785459a);
}

#[test()]
fn test_hasher_sha256_u16() {
    use ::assert::assert;
    let mut hasher = Hasher::new();
    0_u16.hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x96a296d224f285c67bee93c30f8a309157f0daa35dc5b87e410b78630a09cfc7);

    let mut hasher = Hasher::new();
    1_u16.hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0xb413f47d13ee2fe6c845b2ee141af81de858df4ec549a58b7970bb96645bc8d2);
}

#[test()]
fn test_hasher_sha256_u32() {
    use ::assert::assert;
    let mut hasher = Hasher::new();
    0_u32.hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0xdf3f619804a92fdb4057192dc43dd748ea778adc52bc498ce80524c014b81119);

    let mut hasher = Hasher::new();
    1_u32.hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0xb40711a88c7039756fb8a73827eabe2c0fe5a0346ca7e0a104adc0fc764f528d);
}

#[test()]
fn test_hasher_sha256_u64() {
    use ::assert::assert;
    let mut hasher = Hasher::new();
    0_u64.hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0xaf5570f5a1810b7af78caf4bc70a660f0df51e42baf91d4de5b2328de0e83dfc);

    let mut hasher = Hasher::new();
    1_u64.hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0xcd2662154e6d76b2b2b92e70c0cac3ccf534f9b74eb5b89819ec509083d00a50);
}

#[test()]
fn test_hasher_sha256_b256() {
    use ::assert::assert;
    let mut hasher = Hasher::new();
    0x0000000000000000000000000000000000000000000000000000000000000000.hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);

    let mut hasher = Hasher::new();
    0x0000000000000000000000000000000000000000000000000000000000000001.hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0xec4916dd28fc4c10d78e287ca5d9cc51ee1ae73cbfde08c6b37324cbfaac8bc5);
}

#[test()]
fn test_hasher_sha256_bool() {
    use ::assert::assert;
    let mut hasher = Hasher::new();
    false.hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d);

    let mut hasher = Hasher::new();
    true.hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x4bf5122f344554c53bde2ebb8cd2b7e3d1600ad631c385a5d7cce23c7785459a);
}