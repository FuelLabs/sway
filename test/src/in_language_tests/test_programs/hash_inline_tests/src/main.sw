library;

use std::{bytes::Bytes, hash::{Hash, Hasher, keccak256, sha256, sha256_str_array}};

#[test()]
fn hash_hasher_write_str() {
    let mut hasher = Hasher::new();
    hasher.write_str("test");
    let sha256 = hasher.sha256();
    assert(sha256 == 0x9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08);

    let mut hasher = Hasher::new();
    hasher.write_str("Fastest Modular Execution Layer!");
    let sha256 = hasher.sha256();
    assert(sha256 == 0x4a3cd7c8b44dbf7941e55179425f746adeaa97fe2d99b571fffee78e9b41743c);
}

#[test()]
fn hash_hasher_keccak256_str() {
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
fn hash_hasher_write_str_array() {
    let mut hasher = Hasher::new();
    hasher.write_str_array(__to_str_array("test"));
    let sha256 = hasher.sha256();
    assert(sha256 == 0x9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08);

    let mut hasher = Hasher::new();
    hasher.write_str_array(__to_str_array("Fastest Modular Execution Layer!"));
    let sha256 = hasher.sha256();
    assert(sha256 == 0x4a3cd7c8b44dbf7941e55179425f746adeaa97fe2d99b571fffee78e9b41743c);
}

#[test()]
fn hash_hasher_keccak256_str_array() {
    let mut hasher = Hasher::new();
    hasher.write_str_array(__to_str_array("test"));
    let keccak256 = hasher.keccak256();
    assert(keccak256 == 0x9c22ff5f21f0b81b113e63f7db6da94fedef11b2119b4088b89664fb9a3cb658);

    let mut hasher = Hasher::new();
    hasher.write_str_array(__to_str_array("Fastest Modular Execution Layer!"));
    let keccak256 = hasher.keccak256();
    assert(keccak256 == 0xab8e83e041e001bcf797c9cc7d6bc472bfdb8c736bab7999f13b7c26f48c354f);
}

// The hashes for the following test can be obtained in Rust by running the following script:
// https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=a2d83e9ea48b35a3e991c904c3451ed5
#[test()]
fn hash_hasher_sha256_u8() {
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
fn hash_hasher_keccak256_u8() {
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
fn hash_hasher_sha256_u16() {
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
fn hash_hasher_keccak256_u16() {
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
fn hash_hasher_sha256_u32() {
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
fn hash_hasher_keccak256_u32() {
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
fn hash_hasher_sha256_u64() {
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
fn hash_hasher_keccak256_u64() {
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
fn hash_hasher_sha256_b256() {
    let mut hasher = Hasher::new();
    0x0000000000000000000000000000000000000000000000000000000000000000
        .hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);

    let mut hasher = Hasher::new();
    0x0000000000000000000000000000000000000000000000000000000000000001
        .hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0xec4916dd28fc4c10d78e287ca5d9cc51ee1ae73cbfde08c6b37324cbfaac8bc5);
}

#[test()]
fn hash_hasher_keccak256_b256() {
    let mut hasher = Hasher::new();
    0x0000000000000000000000000000000000000000000000000000000000000000
        .hash(hasher);
    let keccak256 = hasher.keccak256();
    assert(keccak256 == 0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563);

    let mut hasher = Hasher::new();
    0x0000000000000000000000000000000000000000000000000000000000000001
        .hash(hasher);
    let keccak256 = hasher.keccak256();
    assert(keccak256 == 0xb10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6);
}

#[test]
fn hash_hasher_sha256_u256() {
    let mut hasher = Hasher::new();
    0x0000000000000000000000000000000000000000000000000000000000000000_u256
        .hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);

    let mut hasher = Hasher::new();
    0x0000000000000000000000000000000000000000000000000000000000000001_u256
        .hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0xec4916dd28fc4c10d78e287ca5d9cc51ee1ae73cbfde08c6b37324cbfaac8bc5);
}

#[test]
fn hash_hasher_keccak256_u256() {
    let mut hasher = Hasher::new();
    0x0000000000000000000000000000000000000000000000000000000000000000_u256
        .hash(hasher);
    let keccak256 = hasher.keccak256();
    assert(keccak256 == 0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563);

    let mut hasher = Hasher::new();
    0x0000000000000000000000000000000000000000000000000000000000000001_u256
        .hash(hasher);
    let keccak256 = hasher.keccak256();
    assert(keccak256 == 0xb10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6);
}

#[test()]
fn hash_hasher_sha256_bool() {
    let mut hasher = Hasher::new();
    false.hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d);

    let mut hasher = Hasher::new();
    true.hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x4bf5122f344554c53bde2ebb8cd2b7e3d1600ad631c385a5d7cce23c7785459a);
}

#[test()]
fn hash_hasher_keccak256_bool() {
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
fn hash_hasher_sha256_bytes() {
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

#[test]
fn hash_hasher_keccak256_bytes() {
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

#[test]
fn hash_hasher_sha256_3_tuple() {
    let mut hasher = Hasher::new();
    (0_u64, 0_u64, 0_u64).hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x9d908ecfb6b256def8b49a7c504e6c889c4b0e41fe6ce3e01863dd7b61a20aa0);

    let mut hasher = Hasher::new();
    (1_u64, 1_u64, 1_u64).hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0xf3dd2c58f4b546018d9a5e147e195b7744eee27b76cae299dad63f221173cca0);
}

#[test]
fn hash_hasher_sha256_4_tuple() {
    let mut hasher = Hasher::new();
    (0_u64, 0_u64, 0_u64, 0_u64).hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);

    let mut hasher = Hasher::new();
    (1_u64, 1_u64, 1_u64, 1_u64).hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x696547da2108716208569c8d60e78fcb423e7ad45cb8c700eeda8a8805bf2571);
}

#[test]
fn hash_hasher_sha256_5_tuple() {
    let mut hasher = Hasher::new();
    (0_u64, 0_u64, 0_u64, 0_u64, 0_u64).hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x2c34ce1df23b838c5abf2a7f6437cca3d3067ed509ff25f11df6b11b582b51eb);

    let mut hasher = Hasher::new();
    (1_u64, 1_u64, 1u64, 1_u64, 1_u64).hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x7bf87db15ea1fff61e936a88ff181b511e66b22417ed270ebb90c298c2088c10);
}

#[test]
fn hash_hasher_sha256_1_array() {
    let mut hasher = Hasher::new();
    [0_u64].hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0xaf5570f5a1810b7af78caf4bc70a660f0df51e42baf91d4de5b2328de0e83dfc);

    let mut hasher = Hasher::new();
    [1_u64].hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0xcd2662154e6d76b2b2b92e70c0cac3ccf534f9b74eb5b89819ec509083d00a50);
}

#[test]
fn hash_hasher_sha256_2_array() {
    let mut hasher = Hasher::new();
    [0_u64, 0_u64].hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x374708fff7719dd5979ec875d56cd2286f6d3cf7ec317a3b25632aab28ec37bb);

    let mut hasher = Hasher::new();
    [1_u64, 1_u64].hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x532deabf88729cb43995ab5a9cd49bf9b90a079904dc0645ecda9e47ce7345a9);
}

#[test]
fn hash_hasher_sha256_4_array() {
    let mut hasher = Hasher::new();
    [0_u64, 0_u64, 0_u64, 0_u64].hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);

    let mut hasher = Hasher::new();
    [1_u64, 1_u64, 1_u64, 1_u64].hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x696547da2108716208569c8d60e78fcb423e7ad45cb8c700eeda8a8805bf2571);
}

#[test]
fn hash_hasher_sha256_5_array() {
    let mut hasher = Hasher::new();
    [0_u64, 0_u64, 0_u64, 0_u64, 0_u64].hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x2c34ce1df23b838c5abf2a7f6437cca3d3067ed509ff25f11df6b11b582b51eb);

    let mut hasher = Hasher::new();
    [1_u64, 1_u64, 1_u64, 1_u64, 1_u64].hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x7bf87db15ea1fff61e936a88ff181b511e66b22417ed270ebb90c298c2088c10);
}

#[test]
fn hash_hasher_sha256_6_array() {
    let mut hasher = Hasher::new();
    [0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64].hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x17b0761f87b081d5cf10757ccc89f12be355c70e2e29df288b65b30710dcbcd1);

    let mut hasher = Hasher::new();
    [1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64].hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x9cd65c79280fcb0d834da54ea98364d11439ec21e106447abcee2893765809a4);
}

#[test]
fn hash_hasher_sha256_8_array() {
    let mut hasher = Hasher::new();
    [0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64]
        .hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0xf5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b);

    let mut hasher = Hasher::new();
    [1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64]
        .hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x794dde2d7e1d63dc28474122bd094bd35499447b3764dbf6cdf7c75ca73918dc);
}

#[test]
fn hash_hasher_sha256_9_array() {
    let mut hasher = Hasher::new();
    [0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64]
        .hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x834a709ba2534ebe3ee1397fd4f7bd288b2acc1d20a08d6c862dcd99b6f04400);

    let mut hasher = Hasher::new();
    [1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64]
        .hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0xe62386a1ec5b8fd0ece7344a7cae775d73179cfc0950c4fdeed26c7e8944e795);
}

#[test]
fn hash_hasher_sha256_10_array() {
    let mut hasher = Hasher::new();
    [0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64]
        .hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x5b6fb58e61fa475939767d68a446f97f1bff02c0e5935a3ea8bb51e6515783d8);

    let mut hasher = Hasher::new();
    [1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64]
        .hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x5f80cf4c3ec64f652ea4ba4db7ea12896224546bd2ed4dd2032a8ce12fde16f9);
}

#[test]
fn hash_hasher_sha256_vec() {
    let mut vec = Vec::<u64>::new();
    let mut i = 0;
    while i < 10 {
        vec.push(0_u64);
        i += 1;
    }

    let mut hasher = Hasher::new();
    vec.hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x5b6fb58e61fa475939767d68a446f97f1bff02c0e5935a3ea8bb51e6515783d8);

    let mut vec = Vec::<u64>::new();
    let mut i = 0;
    while i < 10 {
        vec.push(1_u64);
        i += 1;
    }

    let mut hasher = Hasher::new();
    vec.hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x5f80cf4c3ec64f652ea4ba4db7ea12896224546bd2ed4dd2032a8ce12fde16f9);

    let mut vec = Vec::<(u64, Vec<u64>)>::new();
    let mut inner_vec = Vec::<u64>::new();
    let mut i = 0;
    while i < 9 {
        inner_vec.push(0_u64);
        i += 1;
    }
    vec.push((0_u64, inner_vec));

    let mut hasher = Hasher::new();
    vec.hash(hasher);
    let sha256 = hasher.sha256();
    assert(sha256 == 0x5b6fb58e61fa475939767d68a446f97f1bff02c0e5935a3ea8bb51e6515783d8);
}

#[test()]
fn hash_sha256() {
    let digest = sha256(0_u64);
    assert(digest == 0xaf5570f5a1810b7af78caf4bc70a660f0df51e42baf91d4de5b2328de0e83dfc);

    let digest = sha256(1_u64);
    assert(digest == 0xcd2662154e6d76b2b2b92e70c0cac3ccf534f9b74eb5b89819ec509083d00a50);
}

#[test()]
fn hash_sha256_str_array() {
    let digest = sha256_str_array(__to_str_array("test"));
    assert(digest == 0x9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08);

    let digest = sha256_str_array(__to_str_array("Fastest Modular Execution Layer!"));
    assert(digest == 0x4a3cd7c8b44dbf7941e55179425f746adeaa97fe2d99b571fffee78e9b41743c);
}

#[test()]
fn hash_keccak256() {
    let digest = keccak256(0_u64);
    assert(digest == 0x011b4d03dd8c01f1049143cf9c4c817e4b167f1d1b83e5c6f0f10d89ba1e7bce);

    let digest = keccak256(1_u64);
    assert(digest == 0x6c31fc15422ebad28aaf9089c306702f67540b53c7eea8b7d2941044b027100f);
}
