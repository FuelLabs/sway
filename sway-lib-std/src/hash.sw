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
        let mut result_buffer = b256::min();
        asm(hash: result_buffer, ptr: self.bytes.buf.ptr, bytes: self.bytes.len) {
            s256 hash ptr bytes;
            hash: b256
        }
    }

    pub fn keccak256(self) -> b256 {
        let mut result_buffer = b256::min();
        asm(hash: result_buffer, ptr: self.bytes.buf.ptr, bytes: self.bytes.len) {
            k256 hash ptr bytes;
            hash: b256
        }
    }
}

impl Hasher {
    /// Writes a single `str` into this hasher.
    pub fn write_str(ref mut self, s: str) {
        let str_size = s.len();
        let str_ptr = s.as_ptr();

        let mut bytes = Bytes::with_capacity(str_size);
        bytes.len = str_size;

        str_ptr.copy_bytes_to(bytes.buf.ptr(), str_size);
        self.write(bytes);
    }

    #[inline(never)]
    pub fn write_str_array<S>(ref mut self, s: S) {
        __assert_is_str_array::<S>();
        let str_size = __size_of_str_array::<S>();
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

impl Hash for Bytes {
    fn hash(self, ref mut state: Hasher) {
        state.write(self);
    }
}

impl Hash for str {
    fn hash(self, ref mut state: Hasher) {
        state.write_str(self);
    }
}

impl<A, B> Hash for (A, B) where A: Hash, B: Hash  {
    #[inline(never)]
    fn hash(self, ref mut state: Hasher) {
        self.0.hash(state);
        self.1.hash(state);
    }
}

impl<A, B, C> Hash for (A, B, C) where A: Hash, B: Hash, C: Hash {
    fn hash(self, ref mut state: Hasher) {
        self.0.hash(state);
        self.1.hash(state);
        self.2.hash(state);
    }
}

impl<A, B, C, D> Hash for (A, B, C, D) where A: Hash, B: Hash, C: Hash, D: Hash {
    fn hash(self, ref mut state: Hasher) {
        self.0.hash(state);
        self.1.hash(state);
        self.2.hash(state);
        self.3.hash(state);
    }
}

impl<A, B, C, D, E> Hash for (A, B, C, D, E) where A: Hash, B: Hash, C: Hash, D: Hash, E: Hash {
    fn hash(self, ref mut state: Hasher) {
        self.0.hash(state);
        self.1.hash(state);
        self.2.hash(state);
        self.3.hash(state);
        self.4.hash(state);
    }
}

impl<T> Hash for [T; 1] where T: Hash {
    fn hash(self, ref mut state: Hasher) {
        self[0].hash(state);
    }
}

impl<T> Hash for [T; 2] where T: Hash {
    fn hash(self, ref mut state: Hasher) {
        self[0].hash(state);
        self[1].hash(state);
    }
}

impl<T> Hash for [T; 3] where T: Hash {
    fn hash(self, ref mut state: Hasher) {
        self[0].hash(state);
        self[1].hash(state);
        self[2].hash(state);
    }
}

impl<T> Hash for [T; 4] where T: Hash {
    fn hash(self, ref mut state: Hasher) {
        self[0].hash(state);
        self[1].hash(state);
        self[2].hash(state);
        self[3].hash(state);
    }
}

impl<T> Hash for [T; 5] where T: Hash {
    fn hash(self, ref mut state: Hasher) {
        self[0].hash(state);
        self[1].hash(state);
        self[2].hash(state);
        self[3].hash(state);
        self[4].hash(state);
    }
}

impl<T> Hash for [T; 6] where T: Hash {
    fn hash(self, ref mut state: Hasher) {
        self[0].hash(state);
        self[1].hash(state);
        self[2].hash(state);
        self[3].hash(state);
        self[4].hash(state);
        self[5].hash(state);
    }
}

impl<T> Hash for [T; 7] where T: Hash {
    fn hash(self, ref mut state: Hasher) {
        self[0].hash(state);
        self[1].hash(state);
        self[2].hash(state);
        self[3].hash(state);
        self[4].hash(state);
        self[5].hash(state);
        self[6].hash(state);
    }
}

impl<T> Hash for [T; 8] where T: Hash {
    fn hash(self, ref mut state: Hasher) {
        self[0].hash(state);
        self[1].hash(state);
        self[2].hash(state);
        self[3].hash(state);
        self[4].hash(state);
        self[5].hash(state);
        self[6].hash(state);
        self[7].hash(state);
    }
}

impl<T> Hash for [T; 9] where T: Hash {
    fn hash(self, ref mut state: Hasher) {
        self[0].hash(state);
        self[1].hash(state);
        self[2].hash(state);
        self[3].hash(state);
        self[4].hash(state);
        self[5].hash(state);
        self[6].hash(state);
        self[7].hash(state);
        self[8].hash(state);
    }
}

impl<T> Hash for [T; 10] where T: Hash {
    fn hash(self, ref mut state: Hasher) {
        self[0].hash(state);
        self[1].hash(state);
        self[2].hash(state);
        self[3].hash(state);
        self[4].hash(state);
        self[5].hash(state);
        self[6].hash(state);
        self[7].hash(state);
        self[8].hash(state);
        self[9].hash(state);
    }
}

/// Returns the `SHA-2-256` hash of `param`.
///
/// # Arguments
///
/// * `s`: [T] - The value to be hashed.
///
/// # Returns
///
/// * [b256] - The sha-256 hash of the value.
///
/// # Examples
/// 
/// ```sway
/// use std::hash::*;
///
/// fn foo() {
///     let result = sha256("Fuel");
///     assert(result = 0xa80f942f4112036dfc2da86daf6d2ef6ede3164dd56d1000eb82fa87c992450f);
/// }
/// ```
#[inline(never)]
pub fn sha256<T>(s: T) -> b256 where T: Hash {
    let mut hasher = Hasher::new();
    s.hash(hasher);
    hasher.sha256()
}

/// Returns the `SHA-2-256` hash of `param`.
/// This function is specific for string arrays
///
/// # Examples
/// 
/// ```sway
/// use std::hash::*;
///
/// fn foo() {
///     let result = sha256_str_array(__to_str_array("Fuel"));
///     assert(result = 0xa80f942f4112036dfc2da86daf6d2ef6ede3164dd56d1000eb82fa87c992450f);
/// }
/// ```
#[inline(never)]
pub fn sha256_str_array<S>(param: S) -> b256 {
     __assert_is_str_array::<S>();
    let str_size = __size_of_str_array::<S>();
    let str_ptr = __addr_of(param);
    
    let mut bytes = Bytes::with_capacity(str_size);
    bytes.len = str_size;

    str_ptr.copy_bytes_to(bytes.buf.ptr(), str_size);
    
    let mut hasher = Hasher::new();
    hasher.write(bytes);
    hasher.sha256()
}

/// Returns the `KECCAK-256` hash of `param`.
///
/// # Arguments
///
/// * `s`: [T] - The value to be hashed.
///
/// # Returns
///
/// * [b256] - The keccak-256 hash of the value.
///
/// # Examples
/// 
/// ```sway
/// use std::hash::keccak256;
///
/// fn foo() {
///     let result = keccak256("Fuel");
///     assert(result = 0x4375c8bcdc904e5f51752581202ae9ae2bb6eddf8de05d5567d9a6b0ae4789ad);
/// }
/// ```
#[inline(never)]
pub fn keccak256<T>(s: T) -> b256 where T: Hash {
    let mut hasher = Hasher::new();
    s.hash(hasher);
    hasher.keccak256()
}

// Tests

#[test()]
fn test_hasher_sha256_str_array() {
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

#[test]
fn test_hasher_sha256_bytes() {
    use ::assert::assert;
    let mut hasher = Hasher::new();
    let mut bytes = Bytes::new();
    bytes.push(0u8);
    bytes.hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d);

    let mut hasher = Hasher::new();
    let mut bytes = Bytes::new();
    bytes.push(1u8);
    bytes.hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x4bf5122f344554c53bde2ebb8cd2b7e3d1600ad631c385a5d7cce23c7785459a);
}

#[test()]
fn test_hasher_keccak256_str_array() {
    use ::assert::assert;
    let mut hasher = Hasher::new();
    hasher.write_str("test");
    let keccak256 = hasher.keccak256();
    assert(keccak256 == 0x9c22ff5f21f0b81b113e63f7db6da94fedef11b2119b4088b89664fb9a3cb658);

    let mut hasher = Hasher::new();
    hasher.write_str("Fastest Modular Execution Layer!");
    let keccak256 = hasher.keccak256();
    assert(keccak256 == 0xab8e83e041e001bcf797c9cc7d6bc472bfdb8c736bab7999f13b7c26f48c354f);
}

#[test()]
fn test_hasher_keccak256_u8() {
    use ::assert::assert;
    let mut hasher = Hasher::new();
    0_u8.hash(hasher);
    let keccak256 = hasher.keccak256();
    assert(keccak256 == 0xbc36789e7a1e281436464229828f817d6612f7b477d66591ff96a9e064bcc98a);

    let mut hasher = Hasher::new();
    1_u8.hash(hasher);
    let keccak256 = hasher.keccak256();
    assert(keccak256 == 0x5fe7f977e71dba2ea1a68e21057beebb9be2ac30c6410aa38d4f3fbe41dcffd2);
}

#[test()]
fn test_hasher_keccak256_u16() {
    use ::assert::assert;
    let mut hasher = Hasher::new();
    0_u16.hash(hasher);
    let keccak256 = hasher.keccak256();
    assert(keccak256 == 0x54a8c0ab653c15bfb48b47fd011ba2b9617af01cb45cab344acd57c924d56798);

    let mut hasher = Hasher::new();
    1_u16.hash(hasher);
    let keccak256 = hasher.keccak256();
    assert(keccak256 == 0x49d03a195e239b52779866b33024210fc7dc66e9c2998975c0aa45c1702549d5);
}

#[test()]
fn test_hasher_keccak256_u32() {
    use ::assert::assert;
    let mut hasher = Hasher::new();
    0_u32.hash(hasher);
    let keccak256 = hasher.keccak256();
    assert(keccak256 == 0xe8e77626586f73b955364c7b4bbf0bb7f7685ebd40e852b164633a4acbd3244c);

    let mut hasher = Hasher::new();
    1_u32.hash(hasher);
    let keccak256 = hasher.keccak256();
    assert(keccak256 == 0x51f81bcdfc324a0dff2b5bec9d92e21cbebc4d5e29d3a3d30de3e03fbeab8d7f);
}

#[test()]
fn test_hasher_keccak256_u64() {
    use ::assert::assert;
    let mut hasher = Hasher::new();
    0_u64.hash(hasher);
    let keccak256 = hasher.keccak256();
    assert(keccak256 == 0x011b4d03dd8c01f1049143cf9c4c817e4b167f1d1b83e5c6f0f10d89ba1e7bce);

    let mut hasher = Hasher::new();
    1_u64.hash(hasher);
    let keccak256 = hasher.keccak256();
    assert(keccak256 == 0x6c31fc15422ebad28aaf9089c306702f67540b53c7eea8b7d2941044b027100f);
}

#[test()]
fn test_hasher_keccak256_b256() {
    use ::assert::assert;
    let mut hasher = Hasher::new();
    0x0000000000000000000000000000000000000000000000000000000000000000.hash(hasher);
    let keccak256 = hasher.keccak256();
    assert(keccak256 == 0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563);

    let mut hasher = Hasher::new();
    0x0000000000000000000000000000000000000000000000000000000000000001.hash(hasher);
    let keccak256 = hasher.keccak256();
    assert(keccak256 == 0xb10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6);
}

#[test()]
fn test_hasher_keccak256_bool() {
    use ::assert::assert;
    let mut hasher = Hasher::new();
    false.hash(hasher);
    let keccak256 = hasher.keccak256();
    assert(keccak256 == 0xbc36789e7a1e281436464229828f817d6612f7b477d66591ff96a9e064bcc98a);

    let mut hasher = Hasher::new();
    true.hash(hasher);
    let keccak256 = hasher.keccak256();
    assert(keccak256 == 0x5fe7f977e71dba2ea1a68e21057beebb9be2ac30c6410aa38d4f3fbe41dcffd2);
}

#[test]
fn test_hasher_keccak256_bytes() {
    use ::assert::assert;
    let mut hasher = Hasher::new();
    let mut bytes = Bytes::with_capacity(1);
    bytes.push(0u8);
    bytes.hash(hasher);
    let keccak256 = hasher.keccak256();
    assert(keccak256 == 0xbc36789e7a1e281436464229828f817d6612f7b477d66591ff96a9e064bcc98a);

    let mut hasher = Hasher::new();
    let mut bytes = Bytes::with_capacity(1);
    bytes.push(1u8);
    bytes.hash(hasher);
    let keccak256 = hasher.keccak256();
    assert(keccak256 == 0x5fe7f977e71dba2ea1a68e21057beebb9be2ac30c6410aa38d4f3fbe41dcffd2);
}
