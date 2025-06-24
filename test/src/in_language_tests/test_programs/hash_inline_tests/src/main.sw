library;

use std::address::Address;
use std::asset_id::AssetId;
use std::bytes::Bytes;
use std::b512::B512;
use std::contract_id::ContractId;
use std::crypto::ed25519::Ed25519;
use std::crypto::message::Message;
use std::crypto::public_key::PublicKey;
use std::crypto::point2d::Point2D;
use std::crypto::scalar::Scalar;
use std::crypto::signature::Signature;
use std::crypto::secp256k1::Secp256k1;
use std::crypto::secp256r1::Secp256r1;
use std::time::{Duration, Time};
use std::vm::evm::evm_address::EvmAddress;
use std::hash::*;
use std::identity::Identity;
use std::inputs::Input;
use std::low_level_call::CallParams;
use std::outputs::Output;
use std::string::String;
use std::tx::Transaction;
use std::u128::U128;

// Test `Hasher` methods and `std::hash` functions.

#[test()]
fn hash_hasher_write_str() {
    let mut hasher = Hasher::new();
    hasher.write_str("");
    assert_eq(hasher.sha256(), 0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855);
    assert_eq(hasher.keccak256(), 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470);

    let mut hasher = Hasher::new();
    hasher.write_str("test");
    assert_eq(hasher.sha256(), 0x9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08);
    assert_eq(hasher.keccak256(), 0x9c22ff5f21f0b81b113e63f7db6da94fedef11b2119b4088b89664fb9a3cb658);

    let mut hasher = Hasher::new();
    hasher.write_str("Fastest Modular Execution Layer!");
    assert_eq(hasher.sha256(), 0x4a3cd7c8b44dbf7941e55179425f746adeaa97fe2d99b571fffee78e9b41743c);
    assert_eq(hasher.keccak256(), 0xab8e83e041e001bcf797c9cc7d6bc472bfdb8c736bab7999f13b7c26f48c354f);
}

#[test()]
fn hash_hasher_write_str_array() {
    let mut hasher = Hasher::new();
    hasher.write_str_array(__to_str_array(""));
    assert_eq(hasher.sha256(), 0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855);
    assert_eq(hasher.keccak256(), 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470);

    let mut hasher = Hasher::new();
    hasher.write_str_array(__to_str_array("test"));
    assert_eq(hasher.sha256(), 0x9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08);
    assert_eq(hasher.keccak256(), 0x9c22ff5f21f0b81b113e63f7db6da94fedef11b2119b4088b89664fb9a3cb658);

    let mut hasher = Hasher::new();
    hasher.write_str_array(__to_str_array("Fastest Modular Execution Layer!"));
    assert_eq(hasher.sha256(), 0x4a3cd7c8b44dbf7941e55179425f746adeaa97fe2d99b571fffee78e9b41743c);
    assert_eq(hasher.keccak256(), 0xab8e83e041e001bcf797c9cc7d6bc472bfdb8c736bab7999f13b7c26f48c354f);
}

#[test()]
fn hash_fn_sha256_str_array() {
    let digest = sha256_str_array(__to_str_array(""));
    assert_eq(digest, 0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855);

    let digest = sha256_str_array(__to_str_array("test"));
    assert_eq(digest, 0x9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08);

    let digest = sha256_str_array(__to_str_array("Fastest Modular Execution Layer!"));
    assert_eq(digest, 0x4a3cd7c8b44dbf7941e55179425f746adeaa97fe2d99b571fffee78e9b41743c);
}

// Test `Hash` implementations for all `std` types.
// Standard library types that implement `Hash` trait can be used in cases that
// semantically assume that the hash is deterministic. E.g., as a key in a `StorageMap`.
// These tests ensure that the hash values for these types are stable and deterministic.

// Note that we test `std::hash::sha256` and `std::hash::keccak256` module functions
// next to the `Hasher::sha256` and `Hasher::keccak256`. The reason is that their
// implementations use differently initialized `Hasher` instances, optimized for the
// hashed type size, whereas the `Hasher` used in tests are always initialized with
// `Hasher::new`.

// The hashes used in tests can be obtained in Rust by running the following script:
// https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=cc885f4ba8c7ded1da707909ce38c11b
//
// Note that the **script cannot be executed directly in the Rust Playground**, because
// of the missing dependencies.
//
// To run the script, you need to create a new Rust project, copy the script into the `main.rs` file,
// and add the following dependencies to your `Cargo.toml` file:
// sha2   = "0.10"
// tiny-keccak = { version = "2.0.0", features = ["keccak"] }
// hex    = "0.4"
// bincode = "1.3"
// serde  = { version = "1", features = ["derive"] }
#[test()]
fn hash_u8() {
    let mut hasher = Hasher::new();
    0_u8.hash(hasher);
    assert_eq(hasher.sha256(), 0x6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d);
    assert_eq(sha256(0_u8), 0x6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d);
    assert_eq(hasher.keccak256(), 0xbc36789e7a1e281436464229828f817d6612f7b477d66591ff96a9e064bcc98a);
    assert_eq(keccak256(0_u8), 0xbc36789e7a1e281436464229828f817d6612f7b477d66591ff96a9e064bcc98a);

    let mut hasher = Hasher::new();
    1_u8.hash(hasher);
    assert_eq(hasher.sha256(), 0x4bf5122f344554c53bde2ebb8cd2b7e3d1600ad631c385a5d7cce23c7785459a);
    assert_eq(sha256(1_u8), 0x4bf5122f344554c53bde2ebb8cd2b7e3d1600ad631c385a5d7cce23c7785459a);
    assert_eq(hasher.keccak256(), 0x5fe7f977e71dba2ea1a68e21057beebb9be2ac30c6410aa38d4f3fbe41dcffd2);
    assert_eq(keccak256(1_u8), 0x5fe7f977e71dba2ea1a68e21057beebb9be2ac30c6410aa38d4f3fbe41dcffd2);

    let mut hasher = Hasher::new();
    42_u8.hash(hasher);
    assert_eq(hasher.sha256(), 0x684888c0ebb17f374298b65ee2807526c066094c701bcc7ebbe1c1095f494fc1);
    assert_eq(sha256(42_u8), 0x684888c0ebb17f374298b65ee2807526c066094c701bcc7ebbe1c1095f494fc1);
    assert_eq(hasher.keccak256(), 0x04994f67dc55b09e814ab7ffc8df3686b4afb2bb53e60eae97ef043fe03fb829);
    assert_eq(keccak256(42_u8), 0x04994f67dc55b09e814ab7ffc8df3686b4afb2bb53e60eae97ef043fe03fb829);

    let mut hasher = Hasher::new();
    u8::max().hash(hasher);
    assert_eq(hasher.sha256(), 0xa8100ae6aa1940d0b663bb31cd466142ebbdbd5187131b92d93818987832eb89);
    assert_eq(sha256(u8::max()), 0xa8100ae6aa1940d0b663bb31cd466142ebbdbd5187131b92d93818987832eb89);
    assert_eq(hasher.keccak256(), 0x8b1a944cf13a9a1c08facb2c9e98623ef3254d2ddb48113885c3e8e97fec8db9);
    assert_eq(keccak256(u8::max()), 0x8b1a944cf13a9a1c08facb2c9e98623ef3254d2ddb48113885c3e8e97fec8db9);
}

#[test()]
fn hash_u16() {
    let mut hasher = Hasher::new();
    0_u16.hash(hasher);
    assert_eq(hasher.sha256(), 0x96a296d224f285c67bee93c30f8a309157f0daa35dc5b87e410b78630a09cfc7);
    assert_eq(sha256(0_u16), 0x96a296d224f285c67bee93c30f8a309157f0daa35dc5b87e410b78630a09cfc7);
    assert_eq(hasher.keccak256(), 0x54a8c0ab653c15bfb48b47fd011ba2b9617af01cb45cab344acd57c924d56798);
    assert_eq(keccak256(0_u16), 0x54a8c0ab653c15bfb48b47fd011ba2b9617af01cb45cab344acd57c924d56798);

    let mut hasher = Hasher::new();
    1_u16.hash(hasher);
    assert_eq(hasher.sha256(), 0xb413f47d13ee2fe6c845b2ee141af81de858df4ec549a58b7970bb96645bc8d2);
    assert_eq(sha256(1_u16), 0xb413f47d13ee2fe6c845b2ee141af81de858df4ec549a58b7970bb96645bc8d2);
    assert_eq(hasher.keccak256(), 0x49d03a195e239b52779866b33024210fc7dc66e9c2998975c0aa45c1702549d5);
    assert_eq(keccak256(1_u16), 0x49d03a195e239b52779866b33024210fc7dc66e9c2998975c0aa45c1702549d5);

    let mut hasher = Hasher::new();
    42_u16.hash(hasher);
    assert_eq(hasher.sha256(), 0x587bae728805519c3542d21766295396bd01087b6c47765ae3cadbf679813bbe);
    assert_eq(sha256(42_u16), 0x587bae728805519c3542d21766295396bd01087b6c47765ae3cadbf679813bbe);
    assert_eq(hasher.keccak256(), 0x0643ec401d1673f6c0a7fdf5eb86c0896a7783ad7502e8e08e4b844f204f9bfd);
    assert_eq(keccak256(42_u16), 0x0643ec401d1673f6c0a7fdf5eb86c0896a7783ad7502e8e08e4b844f204f9bfd);

    let mut hasher = Hasher::new();
    u16::max().hash(hasher);
    assert_eq(hasher.sha256(), 0xca2fd00fa001190744c15c317643ab092e7048ce086a243e2be9437c898de1bb);
    assert_eq(sha256(u16::max()), 0xca2fd00fa001190744c15c317643ab092e7048ce086a243e2be9437c898de1bb);
    assert_eq(hasher.keccak256(), 0x06d41322d79dfed27126569cb9a80eb0967335bf2f3316359d2a93c779fcd38a);
    assert_eq(keccak256(u16::max()), 0x06d41322d79dfed27126569cb9a80eb0967335bf2f3316359d2a93c779fcd38a);
}

#[test()]
fn hash_u32() {
    let mut hasher = Hasher::new();
    0_u32.hash(hasher);
    assert_eq(hasher.sha256(), 0xdf3f619804a92fdb4057192dc43dd748ea778adc52bc498ce80524c014b81119);
    assert_eq(sha256(0_u32), 0xdf3f619804a92fdb4057192dc43dd748ea778adc52bc498ce80524c014b81119);
    assert_eq(hasher.keccak256(), 0xe8e77626586f73b955364c7b4bbf0bb7f7685ebd40e852b164633a4acbd3244c);
    assert_eq(keccak256(0_u32), 0xe8e77626586f73b955364c7b4bbf0bb7f7685ebd40e852b164633a4acbd3244c);

    let mut hasher = Hasher::new();
    1_u32.hash(hasher);
    assert_eq(hasher.sha256(), 0xb40711a88c7039756fb8a73827eabe2c0fe5a0346ca7e0a104adc0fc764f528d);
    assert_eq(sha256(1_u32), 0xb40711a88c7039756fb8a73827eabe2c0fe5a0346ca7e0a104adc0fc764f528d);
    assert_eq(hasher.keccak256(), 0x51f81bcdfc324a0dff2b5bec9d92e21cbebc4d5e29d3a3d30de3e03fbeab8d7f);
    assert_eq(keccak256(1_u32), 0x51f81bcdfc324a0dff2b5bec9d92e21cbebc4d5e29d3a3d30de3e03fbeab8d7f);

    let mut hasher = Hasher::new();
    42_u32.hash(hasher);
    assert_eq(hasher.sha256(), 0xae3c8b8d99a39542f78af83dbbb42c81cd94199ec1b5f60a0801063e95842570);
    assert_eq(sha256(42_u32), 0xae3c8b8d99a39542f78af83dbbb42c81cd94199ec1b5f60a0801063e95842570);
    assert_eq(hasher.keccak256(), 0x8ee05353f8b0422de4215090f2a54f3b3f9d13eb2e7e23ef1733da62e7b746de);
    assert_eq(keccak256(42_u32), 0x8ee05353f8b0422de4215090f2a54f3b3f9d13eb2e7e23ef1733da62e7b746de);

    let mut hasher = Hasher::new();
    u32::max().hash(hasher);
    assert_eq(hasher.sha256(), 0xad95131bc0b799c0b1af477fb14fcf26a6a9f76079e48bf090acb7e8367bfd0e);
    assert_eq(sha256(u32::max()), 0xad95131bc0b799c0b1af477fb14fcf26a6a9f76079e48bf090acb7e8367bfd0e);
    assert_eq(hasher.keccak256(), 0x29045a592007d0c246ef02c2223570da9522d0cf0f73282c79a1bc8f0bb2c238);
    assert_eq(keccak256(u32::max()), 0x29045a592007d0c246ef02c2223570da9522d0cf0f73282c79a1bc8f0bb2c238);
}

#[test()]
fn hash_u64() {
    let mut hasher = Hasher::new();
    0_u64.hash(hasher);
    assert_eq(hasher.sha256(), 0xaf5570f5a1810b7af78caf4bc70a660f0df51e42baf91d4de5b2328de0e83dfc);
    assert_eq(sha256(0_u64), 0xaf5570f5a1810b7af78caf4bc70a660f0df51e42baf91d4de5b2328de0e83dfc);
    assert_eq(hasher.keccak256(), 0x011b4d03dd8c01f1049143cf9c4c817e4b167f1d1b83e5c6f0f10d89ba1e7bce);
    assert_eq(keccak256(0_u64), 0x011b4d03dd8c01f1049143cf9c4c817e4b167f1d1b83e5c6f0f10d89ba1e7bce);

    let mut hasher = Hasher::new();
    1_u64.hash(hasher);
    assert_eq(hasher.sha256(), 0xcd2662154e6d76b2b2b92e70c0cac3ccf534f9b74eb5b89819ec509083d00a50);
    assert_eq(sha256(1_u64), 0xcd2662154e6d76b2b2b92e70c0cac3ccf534f9b74eb5b89819ec509083d00a50);
    assert_eq(hasher.keccak256(), 0x6c31fc15422ebad28aaf9089c306702f67540b53c7eea8b7d2941044b027100f);
    assert_eq(keccak256(1_u64), 0x6c31fc15422ebad28aaf9089c306702f67540b53c7eea8b7d2941044b027100f);

    let mut hasher = Hasher::new();
    42_u64.hash(hasher);
    assert_eq(hasher.sha256(), 0xa6bb133cb1e3638ad7b8a3ff0539668e9e56f9b850ef1b2a810f5422eaa6c323);
    assert_eq(sha256(42_u64), 0xa6bb133cb1e3638ad7b8a3ff0539668e9e56f9b850ef1b2a810f5422eaa6c323);
    assert_eq(hasher.keccak256(), 0xc915e80eae100359639667317a39e43392d56b02d9328e8069bb872011b6e63b);
    assert_eq(keccak256(42_u64), 0xc915e80eae100359639667317a39e43392d56b02d9328e8069bb872011b6e63b);

    let mut hasher = Hasher::new();
    u64::max().hash(hasher);
    assert_eq(hasher.sha256(), 0x12a3ae445661ce5dee78d0650d33362dec29c4f82af05e7e57fb595bbbacf0ca);
    assert_eq(sha256(u64::max()), 0x12a3ae445661ce5dee78d0650d33362dec29c4f82af05e7e57fb595bbbacf0ca);
    assert_eq(hasher.keccak256(), 0xad0bfb4b0a66700aeb759d88c315168cc0a11ee99e2a680e548ecf0a464e7daf);
    assert_eq(keccak256(u64::max()), 0xad0bfb4b0a66700aeb759d88c315168cc0a11ee99e2a680e548ecf0a464e7daf);
}

#[test()]
fn hash_u256() {
    let mut hasher = Hasher::new();
    0_u256.hash(hasher);
    assert_eq(hasher.sha256(), 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);
    assert_eq(sha256(0_u256), 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);
    assert_eq(hasher.keccak256(), 0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563);
    assert_eq(keccak256(0_u256), 0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563);

    let mut hasher = Hasher::new();
    1_u256.hash(hasher);
    assert_eq(hasher.sha256(), 0xec4916dd28fc4c10d78e287ca5d9cc51ee1ae73cbfde08c6b37324cbfaac8bc5);
    assert_eq(sha256(1_u256), 0xec4916dd28fc4c10d78e287ca5d9cc51ee1ae73cbfde08c6b37324cbfaac8bc5);
    assert_eq(hasher.keccak256(), 0xb10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6);
    assert_eq(keccak256(1_u256), 0xb10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6);

    let mut hasher = Hasher::new();
    42_u256.hash(hasher);
    assert_eq(hasher.sha256(), 0x0a28e9ffef0073f9a6a674cf57ee77307f38f0f1bebb087888d9011ed0eeefdf);
    assert_eq(sha256(42_u256), 0x0a28e9ffef0073f9a6a674cf57ee77307f38f0f1bebb087888d9011ed0eeefdf);
    assert_eq(hasher.keccak256(), 0xbeced09521047d05b8960b7e7bcc1d1292cf3e4b2a6b63f48335cbde5f7545d2);
    assert_eq(keccak256(42_u256), 0xbeced09521047d05b8960b7e7bcc1d1292cf3e4b2a6b63f48335cbde5f7545d2);

    let mut hasher = Hasher::new();
    u256::max().hash(hasher);
    assert_eq(hasher.sha256(), 0xaf9613760f72635fbdb44a5a0a63c39f12af30f950a6ee5c971be188e89c4051);
    assert_eq(sha256(u256::max()), 0xaf9613760f72635fbdb44a5a0a63c39f12af30f950a6ee5c971be188e89c4051);
    assert_eq(hasher.keccak256(), 0xa9c584056064687e149968cbab758a3376d22aedc6a55823d1b3ecbee81b8fb9);
    assert_eq(keccak256(u256::max()), 0xa9c584056064687e149968cbab758a3376d22aedc6a55823d1b3ecbee81b8fb9);
}

#[test()]
fn hash_b256() {
    let mut hasher = Hasher::new();
    0x0000000000000000000000000000000000000000000000000000000000000000.hash(hasher);
    assert_eq(hasher.sha256(), 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);
    assert_eq(sha256(0x0000000000000000000000000000000000000000000000000000000000000000), 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);
    assert_eq(hasher.keccak256(), 0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563);
    assert_eq(keccak256(0x0000000000000000000000000000000000000000000000000000000000000000), 0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563);

    let mut hasher = Hasher::new();
    0x0000000000000000000000000000000000000000000000000000000000000001.hash(hasher);
    assert_eq(hasher.sha256(), 0xec4916dd28fc4c10d78e287ca5d9cc51ee1ae73cbfde08c6b37324cbfaac8bc5);
    assert_eq(sha256(0x0000000000000000000000000000000000000000000000000000000000000001), 0xec4916dd28fc4c10d78e287ca5d9cc51ee1ae73cbfde08c6b37324cbfaac8bc5);
    assert_eq(hasher.keccak256(), 0xb10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6);
    assert_eq(keccak256(0x0000000000000000000000000000000000000000000000000000000000000001), 0xb10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6);

    let mut hasher = Hasher::new();
    0x000000000000000000000000000000000000000000000000000000000000002a.hash(hasher);
    assert_eq(hasher.sha256(), 0x0a28e9ffef0073f9a6a674cf57ee77307f38f0f1bebb087888d9011ed0eeefdf);
    assert_eq(sha256(0x000000000000000000000000000000000000000000000000000000000000002a), 0x0a28e9ffef0073f9a6a674cf57ee77307f38f0f1bebb087888d9011ed0eeefdf);
    assert_eq(hasher.keccak256(), 0xbeced09521047d05b8960b7e7bcc1d1292cf3e4b2a6b63f48335cbde5f7545d2);
    assert_eq(keccak256(0x000000000000000000000000000000000000000000000000000000000000002a), 0xbeced09521047d05b8960b7e7bcc1d1292cf3e4b2a6b63f48335cbde5f7545d2);

    let mut hasher = Hasher::new();
    0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff.hash(hasher);
    assert_eq(hasher.sha256(), 0xaf9613760f72635fbdb44a5a0a63c39f12af30f950a6ee5c971be188e89c4051);
    assert_eq(sha256(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff), 0xaf9613760f72635fbdb44a5a0a63c39f12af30f950a6ee5c971be188e89c4051);
    assert_eq(hasher.keccak256(), 0xa9c584056064687e149968cbab758a3376d22aedc6a55823d1b3ecbee81b8fb9);
    assert_eq(keccak256(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff), 0xa9c584056064687e149968cbab758a3376d22aedc6a55823d1b3ecbee81b8fb9);
}

#[test()]
fn hash_bool() {
    let mut hasher = Hasher::new();
    false.hash(hasher);
    assert_eq(hasher.sha256(), 0x6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d);
    assert_eq(sha256(false), 0x6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d);
    assert_eq(hasher.keccak256(), 0xbc36789e7a1e281436464229828f817d6612f7b477d66591ff96a9e064bcc98a);
    assert_eq(keccak256(false), 0xbc36789e7a1e281436464229828f817d6612f7b477d66591ff96a9e064bcc98a);

    let mut hasher = Hasher::new();
    true.hash(hasher);
    assert_eq(hasher.sha256(), 0x4bf5122f344554c53bde2ebb8cd2b7e3d1600ad631c385a5d7cce23c7785459a);
    assert_eq(sha256(true), 0x4bf5122f344554c53bde2ebb8cd2b7e3d1600ad631c385a5d7cce23c7785459a);
    assert_eq(hasher.keccak256(), 0x5fe7f977e71dba2ea1a68e21057beebb9be2ac30c6410aa38d4f3fbe41dcffd2);
    assert_eq(keccak256(true), 0x5fe7f977e71dba2ea1a68e21057beebb9be2ac30c6410aa38d4f3fbe41dcffd2);
}

#[cfg(experimental_new_hashing = false)]
#[test()]
fn hash_str() {
    let mut hasher = Hasher::new();
    "".hash(hasher);
    assert_eq(hasher.sha256(), 0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855);
    assert_eq(sha256(""), 0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855);
    assert_eq(hasher.keccak256(), 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470);
    assert_eq(keccak256(""), 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470);

    let mut hasher = Hasher::new();
    "test".hash(hasher);
    assert_eq(hasher.sha256(), 0x9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08);
    assert_eq(sha256("test"), 0x9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08);
    assert_eq(hasher.keccak256(), 0x9c22ff5f21f0b81b113e63f7db6da94fedef11b2119b4088b89664fb9a3cb658);
    assert_eq(keccak256("test"), 0x9c22ff5f21f0b81b113e63f7db6da94fedef11b2119b4088b89664fb9a3cb658);

    let mut hasher = Hasher::new();
    "Fastest Modular Execution Layer!".hash(hasher);
    assert_eq(hasher.sha256(), 0x4a3cd7c8b44dbf7941e55179425f746adeaa97fe2d99b571fffee78e9b41743c);
    assert_eq(sha256("Fastest Modular Execution Layer!"), 0x4a3cd7c8b44dbf7941e55179425f746adeaa97fe2d99b571fffee78e9b41743c);
    assert_eq(hasher.keccak256(), 0xab8e83e041e001bcf797c9cc7d6bc472bfdb8c736bab7999f13b7c26f48c354f);
    assert_eq(keccak256("Fastest Modular Execution Layer!"), 0xab8e83e041e001bcf797c9cc7d6bc472bfdb8c736bab7999f13b7c26f48c354f);
}

#[cfg(experimental_new_hashing = true)]
#[test()]
fn hash_str() {
    let mut hasher = Hasher::new();
    "".hash(hasher);
    assert_eq(hasher.sha256(), 0xaf5570f5a1810b7af78caf4bc70a660f0df51e42baf91d4de5b2328de0e83dfc);
    assert_eq(sha256(""), 0xaf5570f5a1810b7af78caf4bc70a660f0df51e42baf91d4de5b2328de0e83dfc);
    assert_eq(hasher.keccak256(), 0x011b4d03dd8c01f1049143cf9c4c817e4b167f1d1b83e5c6f0f10d89ba1e7bce);
    assert_eq(keccak256(""), 0x011b4d03dd8c01f1049143cf9c4c817e4b167f1d1b83e5c6f0f10d89ba1e7bce);

    let mut hasher = Hasher::new();
    "test".hash(hasher);
    assert_eq(hasher.sha256(), 0x09a7d352412717c7e0b93286eb544f83ddf6da4260b795e90aa44e8e58f5dadd);
    assert_eq(sha256("test"), 0x09a7d352412717c7e0b93286eb544f83ddf6da4260b795e90aa44e8e58f5dadd);
    assert_eq(hasher.keccak256(), 0x7deeee38ddc74b84935b679921e2554392d9228f46f9845e4f379a3a67635ccd);
    assert_eq(keccak256("test"), 0x7deeee38ddc74b84935b679921e2554392d9228f46f9845e4f379a3a67635ccd);

    let mut hasher = Hasher::new();
    "Fastest Modular Execution Layer!".hash(hasher);
    assert_eq(hasher.sha256(), 0x03e88f60c46971ad474fbcc4b8532136a378b140f5eeb2b26cb490dbd10c51e8);
    assert_eq(sha256("Fastest Modular Execution Layer!"), 0x03e88f60c46971ad474fbcc4b8532136a378b140f5eeb2b26cb490dbd10c51e8);
    assert_eq(hasher.keccak256(), 0x61196ca4771dd4c6c645c2f9f14c7a45c64247b05eb81dd9ffd7ebc68b5b2f7c);
    assert_eq(keccak256("Fastest Modular Execution Layer!"), 0x61196ca4771dd4c6c645c2f9f14c7a45c64247b05eb81dd9ffd7ebc68b5b2f7c);
}

#[test()]
fn hash_unit() {
    let mut hasher = Hasher::new();
    ().hash(hasher);
    assert_eq(hasher.sha256(), 0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855);
    assert_eq(sha256(()), 0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855);
    assert_eq(hasher.keccak256(), 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470);
    assert_eq(keccak256(()), 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470);
}

#[test]
fn hash_tuple_1() {
    let mut hasher = Hasher::new();
    (0_u64, ).hash(hasher);
    assert_eq(hasher.sha256(), 0xaf5570f5a1810b7af78caf4bc70a660f0df51e42baf91d4de5b2328de0e83dfc);
    assert_eq(sha256((0_u64, )), 0xaf5570f5a1810b7af78caf4bc70a660f0df51e42baf91d4de5b2328de0e83dfc);
    assert_eq(hasher.keccak256(), 0x011b4d03dd8c01f1049143cf9c4c817e4b167f1d1b83e5c6f0f10d89ba1e7bce);
    assert_eq(keccak256((0_u64, )), 0x011b4d03dd8c01f1049143cf9c4c817e4b167f1d1b83e5c6f0f10d89ba1e7bce);

    let mut hasher = Hasher::new();
    (1_u64, ).hash(hasher);
    assert_eq(hasher.sha256(), 0xcd2662154e6d76b2b2b92e70c0cac3ccf534f9b74eb5b89819ec509083d00a50);
    assert_eq(sha256((1_u64, )), 0xcd2662154e6d76b2b2b92e70c0cac3ccf534f9b74eb5b89819ec509083d00a50);
    assert_eq(hasher.keccak256(), 0x6c31fc15422ebad28aaf9089c306702f67540b53c7eea8b7d2941044b027100f);
    assert_eq(keccak256((1_u64, )), 0x6c31fc15422ebad28aaf9089c306702f67540b53c7eea8b7d2941044b027100f);
}

#[test]
fn hash_tuple_2() {
    let mut hasher = Hasher::new();
    (0_u64, 0_u64).hash(hasher);
    assert_eq(hasher.sha256(), 0x374708fff7719dd5979ec875d56cd2286f6d3cf7ec317a3b25632aab28ec37bb);
    assert_eq(sha256((0_u64, 0_u64)), 0x374708fff7719dd5979ec875d56cd2286f6d3cf7ec317a3b25632aab28ec37bb);
    assert_eq(hasher.keccak256(), 0xf490de2920c8a35fabeb13208852aa28c76f9be9b03a4dd2b3c075f7a26923b4);
    assert_eq(keccak256((0_u64, 0_u64)), 0xf490de2920c8a35fabeb13208852aa28c76f9be9b03a4dd2b3c075f7a26923b4);

    let mut hasher = Hasher::new();
    (1_u64, 1_u64).hash(hasher);
    assert_eq(hasher.sha256(), 0x532deabf88729cb43995ab5a9cd49bf9b90a079904dc0645ecda9e47ce7345a9);
    assert_eq(sha256((1_u64, 1_u64)), 0x532deabf88729cb43995ab5a9cd49bf9b90a079904dc0645ecda9e47ce7345a9);
    assert_eq(hasher.keccak256(), 0x5dd50243f81eaa0bd39ace71862b46f2054c3ea1c2b69a79093b5795061e3851);
    assert_eq(keccak256((1_u64, 1_u64)), 0x5dd50243f81eaa0bd39ace71862b46f2054c3ea1c2b69a79093b5795061e3851);
}

#[test]
fn hash_tuple_3() {
    let mut hasher = Hasher::new();
    (0_u64, 0_u64, 0_u64).hash(hasher);
    assert_eq(hasher.sha256(), 0x9d908ecfb6b256def8b49a7c504e6c889c4b0e41fe6ce3e01863dd7b61a20aa0);
    assert_eq(sha256((0_u64, 0_u64, 0_u64)), 0x9d908ecfb6b256def8b49a7c504e6c889c4b0e41fe6ce3e01863dd7b61a20aa0);
    assert_eq(hasher.keccak256(), 0x827b659bbda2a0bdecce2c91b8b68462545758f3eba2dbefef18e0daf84f5ccd);
    assert_eq(keccak256((0_u64, 0_u64, 0_u64)), 0x827b659bbda2a0bdecce2c91b8b68462545758f3eba2dbefef18e0daf84f5ccd);

    let mut hasher = Hasher::new();
    (1_u64, 1_u64, 1_u64).hash(hasher);
    assert_eq(hasher.sha256(), 0xf3dd2c58f4b546018d9a5e147e195b7744eee27b76cae299dad63f221173cca0);
    assert_eq(sha256((1_u64, 1_u64, 1_u64)), 0xf3dd2c58f4b546018d9a5e147e195b7744eee27b76cae299dad63f221173cca0);
    assert_eq(hasher.keccak256(), 0xc9385a4b1b6112f4dca587451a622a8dad8846adb4e78b90124930eecf5c2830);
    assert_eq(keccak256((1_u64, 1_u64, 1_u64)), 0xc9385a4b1b6112f4dca587451a622a8dad8846adb4e78b90124930eecf5c2830);
}

#[test]
fn hash_tuple_4() {
    let mut hasher = Hasher::new();
    (0_u64, 0_u64, 0_u64, 0_u64).hash(hasher);
    assert_eq(hasher.sha256(), 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);
    assert_eq(sha256((0_u64, 0_u64, 0_u64, 0_u64)), 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);
    assert_eq(hasher.keccak256(), 0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563);
    assert_eq(keccak256((0_u64, 0_u64, 0_u64, 0_u64)), 0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563);

    let mut hasher = Hasher::new();
    (1_u64, 1_u64, 1_u64, 1_u64).hash(hasher);
    assert_eq(hasher.sha256(), 0x696547da2108716208569c8d60e78fcb423e7ad45cb8c700eeda8a8805bf2571);
    assert_eq(sha256((1_u64, 1_u64, 1_u64, 1_u64)), 0x696547da2108716208569c8d60e78fcb423e7ad45cb8c700eeda8a8805bf2571);
    assert_eq(hasher.keccak256(), 0x4861a1848ac650fbed0906f372ea2ce38824a879872c49eb27da3b5b2d2ee868);
    assert_eq(keccak256((1_u64, 1_u64, 1_u64, 1_u64)), 0x4861a1848ac650fbed0906f372ea2ce38824a879872c49eb27da3b5b2d2ee868);
}

#[test]
fn hash_tuple_5() {
    let mut hasher = Hasher::new();
    (0_u64, 0_u64, 0_u64, 0_u64, 0_u64).hash(hasher);
    assert_eq(hasher.sha256(), 0x2c34ce1df23b838c5abf2a7f6437cca3d3067ed509ff25f11df6b11b582b51eb);
    assert_eq(sha256((0_u64, 0_u64, 0_u64, 0_u64, 0_u64)), 0x2c34ce1df23b838c5abf2a7f6437cca3d3067ed509ff25f11df6b11b582b51eb);
    assert_eq(hasher.keccak256(), 0xdaa77426c30c02a43d9fba4e841a6556c524d47030762eb14dc4af897e605d9b);
    assert_eq(keccak256((0_u64, 0_u64, 0_u64, 0_u64, 0_u64)), 0xdaa77426c30c02a43d9fba4e841a6556c524d47030762eb14dc4af897e605d9b);

    let mut hasher = Hasher::new();
    (1_u64, 1_u64, 1u64, 1_u64, 1_u64).hash(hasher);
    assert_eq(hasher.sha256(), 0x7bf87db15ea1fff61e936a88ff181b511e66b22417ed270ebb90c298c2088c10);
    assert_eq(sha256((1_u64, 1_u64, 1u64, 1_u64, 1_u64)), 0x7bf87db15ea1fff61e936a88ff181b511e66b22417ed270ebb90c298c2088c10);
    assert_eq(hasher.keccak256(), 0x7755da38b585018ea81bdf1a67e2c7de3014469999d4c11eb8c2d99c6285df6c);
    assert_eq(keccak256((1_u64, 1_u64, 1u64, 1_u64, 1_u64)), 0x7755da38b585018ea81bdf1a67e2c7de3014469999d4c11eb8c2d99c6285df6c);
}

#[cfg(experimental_new_hashing = false)]
#[test()]
fn hash_array_empty() {
    let mut hasher = Hasher::new();
    let empty_array: [u64; 0] = [];
    empty_array.hash(hasher);
    assert_eq(hasher.sha256(), 0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855);
    assert_eq(sha256(empty_array), 0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855);
    assert_eq(hasher.keccak256(), 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470);
    assert_eq(keccak256(empty_array), 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470);
}

#[cfg(experimental_new_hashing = true)]
#[test()]
fn hash_array_empty() {
    let mut hasher = Hasher::new();
    let empty_array: [u64; 0] = [];
    empty_array.hash(hasher);
    assert_eq(hasher.sha256(), 0xaf5570f5a1810b7af78caf4bc70a660f0df51e42baf91d4de5b2328de0e83dfc);
    assert_eq(sha256(empty_array), 0xaf5570f5a1810b7af78caf4bc70a660f0df51e42baf91d4de5b2328de0e83dfc);
    assert_eq(hasher.keccak256(), 0x011b4d03dd8c01f1049143cf9c4c817e4b167f1d1b83e5c6f0f10d89ba1e7bce);
    assert_eq(keccak256(empty_array), 0x011b4d03dd8c01f1049143cf9c4c817e4b167f1d1b83e5c6f0f10d89ba1e7bce);
}

#[cfg(experimental_new_hashing = false)]
#[test]
fn hash_array_1() {
    let mut hasher = Hasher::new();
    [0_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0xaf5570f5a1810b7af78caf4bc70a660f0df51e42baf91d4de5b2328de0e83dfc);
    assert_eq(sha256([0_u64]), 0xaf5570f5a1810b7af78caf4bc70a660f0df51e42baf91d4de5b2328de0e83dfc);
    assert_eq(hasher.keccak256(), 0x011b4d03dd8c01f1049143cf9c4c817e4b167f1d1b83e5c6f0f10d89ba1e7bce);
    assert_eq(keccak256([0_u64]), 0x011b4d03dd8c01f1049143cf9c4c817e4b167f1d1b83e5c6f0f10d89ba1e7bce);

    let mut hasher = Hasher::new();
    [1_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0xcd2662154e6d76b2b2b92e70c0cac3ccf534f9b74eb5b89819ec509083d00a50);
    assert_eq(sha256([1_u64]), 0xcd2662154e6d76b2b2b92e70c0cac3ccf534f9b74eb5b89819ec509083d00a50);
    assert_eq(hasher.keccak256(), 0x6c31fc15422ebad28aaf9089c306702f67540b53c7eea8b7d2941044b027100f);
    assert_eq(keccak256([1_u64]), 0x6c31fc15422ebad28aaf9089c306702f67540b53c7eea8b7d2941044b027100f);
}

#[cfg(experimental_new_hashing = true)]
#[test]
fn hash_array_1() {
    let mut hasher = Hasher::new();
    [0_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0x783825822a6f9e62da2190e828e4c9d2576e5977e3a0b3620b092dfb9e9996fa);
    assert_eq(sha256([0_u64]), 0x783825822a6f9e62da2190e828e4c9d2576e5977e3a0b3620b092dfb9e9996fa);
    assert_eq(hasher.keccak256(), 0x1b3dc907f2c72f8fba8e45cebb54b5ff1cb577e25373135a90900c10ed2cdaa3);
    assert_eq(keccak256([0_u64]), 0x1b3dc907f2c72f8fba8e45cebb54b5ff1cb577e25373135a90900c10ed2cdaa3);

    let mut hasher = Hasher::new();
    [1_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0x532deabf88729cb43995ab5a9cd49bf9b90a079904dc0645ecda9e47ce7345a9);
    assert_eq(sha256([1_u64]), 0x532deabf88729cb43995ab5a9cd49bf9b90a079904dc0645ecda9e47ce7345a9);
    assert_eq(hasher.keccak256(), 0x5dd50243f81eaa0bd39ace71862b46f2054c3ea1c2b69a79093b5795061e3851);
    assert_eq(keccak256([1_u64]), 0x5dd50243f81eaa0bd39ace71862b46f2054c3ea1c2b69a79093b5795061e3851);
}

#[cfg(experimental_new_hashing = false)]
#[test]
fn hash_array_2() {
    let mut hasher = Hasher::new();
    [0_u64, 0_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0x374708fff7719dd5979ec875d56cd2286f6d3cf7ec317a3b25632aab28ec37bb);
    assert_eq(sha256([0_u64, 0_u64]), 0x374708fff7719dd5979ec875d56cd2286f6d3cf7ec317a3b25632aab28ec37bb);
    assert_eq(hasher.keccak256(), 0xf490de2920c8a35fabeb13208852aa28c76f9be9b03a4dd2b3c075f7a26923b4);
    assert_eq(keccak256([0_u64, 0_u64]), 0xf490de2920c8a35fabeb13208852aa28c76f9be9b03a4dd2b3c075f7a26923b4);

    let mut hasher = Hasher::new();
    [1_u64, 1_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0x532deabf88729cb43995ab5a9cd49bf9b90a079904dc0645ecda9e47ce7345a9);
    assert_eq(sha256([1_u64, 1_u64]), 0x532deabf88729cb43995ab5a9cd49bf9b90a079904dc0645ecda9e47ce7345a9);
    assert_eq(hasher.keccak256(), 0x5dd50243f81eaa0bd39ace71862b46f2054c3ea1c2b69a79093b5795061e3851);
    assert_eq(keccak256([1_u64, 1_u64]), 0x5dd50243f81eaa0bd39ace71862b46f2054c3ea1c2b69a79093b5795061e3851);
}

#[cfg(experimental_new_hashing = true)]
#[test]
fn hash_array_2() {
    let mut hasher = Hasher::new();
    [0_u64, 0_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0xd63f0d21dab3df7c1d7e65d95b42e3ac52c9c94ecb0c5b85fee1af5fb6062524);
    assert_eq(sha256([0_u64, 0_u64]), 0xd63f0d21dab3df7c1d7e65d95b42e3ac52c9c94ecb0c5b85fee1af5fb6062524);
    assert_eq(hasher.keccak256(), 0x64528e5f7d27818571b16785c39db6dca94467bf1fc1119123fd4b85d38ba912);
    assert_eq(keccak256([0_u64, 0_u64]), 0x64528e5f7d27818571b16785c39db6dca94467bf1fc1119123fd4b85d38ba912);

    let mut hasher = Hasher::new();
    [1_u64, 1_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0x4db0576b7d1ca585ce21ae2a2d3040725e53de5cd9133745967825ab482bfb1e);
    assert_eq(sha256([1_u64, 1_u64]), 0x4db0576b7d1ca585ce21ae2a2d3040725e53de5cd9133745967825ab482bfb1e);
    assert_eq(hasher.keccak256(), 0x477f9223ad6531f8bdefbf9ba32e7af57755d9370c36cf1d515d1d2a8d70408e);
    assert_eq(keccak256([1_u64, 1_u64]), 0x477f9223ad6531f8bdefbf9ba32e7af57755d9370c36cf1d515d1d2a8d70408e);
}

#[cfg(experimental_new_hashing = false)]
#[test]
fn hash_array_3() {
    let mut hasher = Hasher::new();
    [0_u64, 0_u64, 0_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0x9d908ecfb6b256def8b49a7c504e6c889c4b0e41fe6ce3e01863dd7b61a20aa0);
    assert_eq(sha256([0_u64, 0_u64, 0_u64]), 0x9d908ecfb6b256def8b49a7c504e6c889c4b0e41fe6ce3e01863dd7b61a20aa0);
    assert_eq(hasher.keccak256(), 0x827b659bbda2a0bdecce2c91b8b68462545758f3eba2dbefef18e0daf84f5ccd);
    assert_eq(keccak256([0_u64, 0_u64, 0_u64]), 0x827b659bbda2a0bdecce2c91b8b68462545758f3eba2dbefef18e0daf84f5ccd);

    let mut hasher = Hasher::new();
    [1_u64, 1_u64, 1_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0xf3dd2c58f4b546018d9a5e147e195b7744eee27b76cae299dad63f221173cca0);
    assert_eq(sha256([1_u64, 1_u64, 1_u64]), 0xf3dd2c58f4b546018d9a5e147e195b7744eee27b76cae299dad63f221173cca0);
    assert_eq(hasher.keccak256(), 0xc9385a4b1b6112f4dca587451a622a8dad8846adb4e78b90124930eecf5c2830);
    assert_eq(keccak256([1_u64, 1_u64, 1_u64]), 0xc9385a4b1b6112f4dca587451a622a8dad8846adb4e78b90124930eecf5c2830);
}

#[cfg(experimental_new_hashing = true)]
#[test]
fn hash_array_3() {
    let mut hasher = Hasher::new();
    [0_u64, 0_u64, 0_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0x71626eb965307659c44f2e1b567e778bebdf8ac2fbf68ed0f47e79b630f234ac);
    assert_eq(sha256([0_u64, 0_u64, 0_u64]), 0x71626eb965307659c44f2e1b567e778bebdf8ac2fbf68ed0f47e79b630f234ac);
    assert_eq(hasher.keccak256(), 0xcaf45fb5c231c92018351e342b85a4d0244893e1774d7943b7b63f5f7d5efb0d);
    assert_eq(keccak256([0_u64, 0_u64, 0_u64]), 0xcaf45fb5c231c92018351e342b85a4d0244893e1774d7943b7b63f5f7d5efb0d);

    let mut hasher = Hasher::new();
    [1_u64, 1_u64, 1_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0xdfd8414b95821ee567ab939d7c0845d243716bae412949a3bad9e1800ff227f0);
    assert_eq(sha256([1_u64, 1_u64, 1_u64]), 0xdfd8414b95821ee567ab939d7c0845d243716bae412949a3bad9e1800ff227f0);
    assert_eq(hasher.keccak256(), 0xd314e093dae9726c9b7016080047fb7f1c2f806dcf3ee97a2b90bf7a821fc94e);
    assert_eq(keccak256([1_u64, 1_u64, 1_u64]), 0xd314e093dae9726c9b7016080047fb7f1c2f806dcf3ee97a2b90bf7a821fc94e);
}

#[cfg(experimental_new_hashing = false)]
#[test]
fn hash_array_4() {
    let mut hasher = Hasher::new();
    [0_u64, 0_u64, 0_u64, 0_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);
    assert_eq(sha256([0_u64, 0_u64, 0_u64, 0_u64]), 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);
    assert_eq(hasher.keccak256(), 0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563);
    assert_eq(keccak256([0_u64, 0_u64, 0_u64, 0_u64]), 0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563);

    let mut hasher = Hasher::new();
    [1_u64, 1_u64, 1_u64, 1_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0x696547da2108716208569c8d60e78fcb423e7ad45cb8c700eeda8a8805bf2571);
    assert_eq(sha256([1_u64, 1_u64, 1_u64, 1_u64]), 0x696547da2108716208569c8d60e78fcb423e7ad45cb8c700eeda8a8805bf2571);
    assert_eq(hasher.keccak256(), 0x4861a1848ac650fbed0906f372ea2ce38824a879872c49eb27da3b5b2d2ee868);
    assert_eq(keccak256([1_u64, 1_u64, 1_u64, 1_u64]), 0x4861a1848ac650fbed0906f372ea2ce38824a879872c49eb27da3b5b2d2ee868);
}

#[cfg(experimental_new_hashing = true)]
#[test]
fn hash_array_4() {
    let mut hasher = Hasher::new();
    [0_u64, 0_u64, 0_u64, 0_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0x5fa29ed4356903dac2364713c60f57d8472c7dda4a5e08d88a88ad8ea71aed60);
    assert_eq(sha256([0_u64, 0_u64, 0_u64, 0_u64]), 0x5fa29ed4356903dac2364713c60f57d8472c7dda4a5e08d88a88ad8ea71aed60);
    assert_eq(hasher.keccak256(), 0xf602ac09d9228efe41fa2f8d5f72f8a2f5f437eef42ca153d7ca5899627bef40);
    assert_eq(keccak256([0_u64, 0_u64, 0_u64, 0_u64]), 0xf602ac09d9228efe41fa2f8d5f72f8a2f5f437eef42ca153d7ca5899627bef40);

    let mut hasher = Hasher::new();
    [1_u64, 1_u64, 1_u64, 1_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0x62d0128874c916c508459384a11abd9572e67bed4819fd5e138526d74966118a);
    assert_eq(sha256([1_u64, 1_u64, 1_u64, 1_u64]), 0x62d0128874c916c508459384a11abd9572e67bed4819fd5e138526d74966118a);
    assert_eq(hasher.keccak256(), 0x58c178b02b0e11251fd0d2511333caf1046e8d278530e8d80dd00247b55d651a);
    assert_eq(keccak256([1_u64, 1_u64, 1_u64, 1_u64]), 0x58c178b02b0e11251fd0d2511333caf1046e8d278530e8d80dd00247b55d651a);
}

#[cfg(experimental_new_hashing = false)]
#[test]
fn hash_array_5() {
    let mut hasher = Hasher::new();
    [0_u64, 0_u64, 0_u64, 0_u64, 0_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0x2c34ce1df23b838c5abf2a7f6437cca3d3067ed509ff25f11df6b11b582b51eb);
    assert_eq(sha256([0_u64, 0_u64, 0_u64, 0_u64, 0_u64]), 0x2c34ce1df23b838c5abf2a7f6437cca3d3067ed509ff25f11df6b11b582b51eb);
    assert_eq(hasher.keccak256(), 0xdaa77426c30c02a43d9fba4e841a6556c524d47030762eb14dc4af897e605d9b);
    assert_eq(keccak256([0_u64, 0_u64, 0_u64, 0_u64, 0_u64]), 0xdaa77426c30c02a43d9fba4e841a6556c524d47030762eb14dc4af897e605d9b);

    let mut hasher = Hasher::new();
    [1_u64, 1_u64, 1u64, 1_u64, 1_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0x7bf87db15ea1fff61e936a88ff181b511e66b22417ed270ebb90c298c2088c10);
    assert_eq(sha256([1_u64, 1_u64, 1u64, 1_u64, 1_u64]), 0x7bf87db15ea1fff61e936a88ff181b511e66b22417ed270ebb90c298c2088c10);
    assert_eq(hasher.keccak256(), 0x7755da38b585018ea81bdf1a67e2c7de3014469999d4c11eb8c2d99c6285df6c);
    assert_eq(keccak256([1_u64, 1_u64, 1u64, 1_u64, 1_u64]), 0x7755da38b585018ea81bdf1a67e2c7de3014469999d4c11eb8c2d99c6285df6c);
}

#[cfg(experimental_new_hashing = true)]
#[test]
fn hash_array_5() {
    let mut hasher = Hasher::new();
    [0_u64, 0_u64, 0_u64, 0_u64, 0_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0xa2ab9650245ce5d8d90a7520e66257ef75bc894eed13f26d1e39423e26b8ffce);
    assert_eq(sha256([0_u64, 0_u64, 0_u64, 0_u64, 0_u64]), 0xa2ab9650245ce5d8d90a7520e66257ef75bc894eed13f26d1e39423e26b8ffce);
    assert_eq(hasher.keccak256(), 0xededdfe4352a415dcae3bc34c4c59ee5d219a0ca7aef7d6f9a6a43b18b998c6a);
    assert_eq(keccak256([0_u64, 0_u64, 0_u64, 0_u64, 0_u64]), 0xededdfe4352a415dcae3bc34c4c59ee5d219a0ca7aef7d6f9a6a43b18b998c6a);

    let mut hasher = Hasher::new();
    [1_u64, 1_u64, 1u64, 1_u64, 1_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0x7f9e6b1f808270ba0b1b0d73f45223847388c0e11774e359c548da9b0e9ce052);
    assert_eq(sha256([1_u64, 1_u64, 1u64, 1_u64, 1_u64]), 0x7f9e6b1f808270ba0b1b0d73f45223847388c0e11774e359c548da9b0e9ce052);
    assert_eq(hasher.keccak256(), 0x0626c2dc90e5fe0341477c5d8e286abbf4b164610d8613068ee5b022cac8dd88);
    assert_eq(keccak256([1_u64, 1_u64, 1u64, 1_u64, 1_u64]), 0x0626c2dc90e5fe0341477c5d8e286abbf4b164610d8613068ee5b022cac8dd88);
}

#[cfg(experimental_new_hashing = false)]
#[test]
fn hash_array_6() {
    let mut hasher = Hasher::new();
    [0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0x17b0761f87b081d5cf10757ccc89f12be355c70e2e29df288b65b30710dcbcd1);
    assert_eq(sha256([0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64]), 0x17b0761f87b081d5cf10757ccc89f12be355c70e2e29df288b65b30710dcbcd1);
    assert_eq(hasher.keccak256(), 0xc980e59163ce244bb4bb6211f48c7b46f88a4f40943e84eb99bdc41e129bd293);
    assert_eq(keccak256([0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64]), 0xc980e59163ce244bb4bb6211f48c7b46f88a4f40943e84eb99bdc41e129bd293);

    let mut hasher = Hasher::new();
    [1_u64, 1_u64, 1u64, 1_u64, 1_u64, 1_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0x9cd65c79280fcb0d834da54ea98364d11439ec21e106447abcee2893765809a4);
    assert_eq(sha256([1_u64, 1_u64, 1u64, 1_u64, 1_u64, 1_u64]), 0x9cd65c79280fcb0d834da54ea98364d11439ec21e106447abcee2893765809a4);
    assert_eq(hasher.keccak256(), 0xe25b47d30a6a4dc767566eb44c1ca3bb814bc7d05336e407cb7e4fcbb62dfd51);
    assert_eq(keccak256([1_u64, 1_u64, 1u64, 1_u64, 1_u64, 1_u64]), 0xe25b47d30a6a4dc767566eb44c1ca3bb814bc7d05336e407cb7e4fcbb62dfd51);
}

#[cfg(experimental_new_hashing = true)]
#[test]
fn hash_array_6() {
    let mut hasher = Hasher::new();
    [0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0x1f504930ac8bc96ad1764ccb2b2a4552c3f1f94f18a667a186655911db8d3173);
    assert_eq(sha256([0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64]), 0x1f504930ac8bc96ad1764ccb2b2a4552c3f1f94f18a667a186655911db8d3173);
    assert_eq(hasher.keccak256(), 0x0ff7bd31058a483ccaa3778b109a60801d99b9243d4c595cef7c56f0708e032b);
    assert_eq(keccak256([0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64]), 0x0ff7bd31058a483ccaa3778b109a60801d99b9243d4c595cef7c56f0708e032b);

    let mut hasher = Hasher::new();
    [1_u64, 1_u64, 1u64, 1_u64, 1_u64, 1_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0xb6437f1d6c0d4b164e2b5ee05d63df4a1eb38a7c85ee6765f604d18329758fe0);
    assert_eq(sha256([1_u64, 1_u64, 1u64, 1_u64, 1_u64, 1_u64]), 0xb6437f1d6c0d4b164e2b5ee05d63df4a1eb38a7c85ee6765f604d18329758fe0);
    assert_eq(hasher.keccak256(), 0x6d582e20ceb9fce3d134f283602e7f71fc03397ef5c28a7a7b5e3bbce2dd840c);
    assert_eq(keccak256([1_u64, 1_u64, 1u64, 1_u64, 1_u64, 1_u64]), 0x6d582e20ceb9fce3d134f283602e7f71fc03397ef5c28a7a7b5e3bbce2dd840c);
}

#[cfg(experimental_new_hashing = false)]
#[test]
fn hash_array_7() {
    let mut hasher = Hasher::new();
    [0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0xd4817aa5497628e7c77e6b606107042bbba3130888c5f47a375e6179be789fbb);
    assert_eq(sha256([0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64]), 0xd4817aa5497628e7c77e6b606107042bbba3130888c5f47a375e6179be789fbb);
    assert_eq(hasher.keccak256(), 0x660b057b36925d4a0da5bf6588b4c64cff7f27ee34e9c90b052829bf8e2a3168);
    assert_eq(keccak256([0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64]), 0x660b057b36925d4a0da5bf6588b4c64cff7f27ee34e9c90b052829bf8e2a3168);

    let mut hasher = Hasher::new();
    [1_u64, 1_u64, 1u64, 1_u64, 1_u64, 1_u64, 1_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0x68ac8a4116c57147cbdf1560d0aa00bd087e8c3ca484ff219cf85ac2c3249a7b);
    assert_eq(sha256([1_u64, 1_u64, 1u64, 1_u64, 1_u64, 1_u64, 1_u64]), 0x68ac8a4116c57147cbdf1560d0aa00bd087e8c3ca484ff219cf85ac2c3249a7b);
    assert_eq(hasher.keccak256(), 0x4b085430c7e79e4862c111c5b93c007a7c264c60726a71c85fd5cf48927b873a);
    assert_eq(keccak256([1_u64, 1_u64, 1u64, 1_u64, 1_u64, 1_u64, 1_u64]), 0x4b085430c7e79e4862c111c5b93c007a7c264c60726a71c85fd5cf48927b873a);
}

#[cfg(experimental_new_hashing = true)]
#[test]
fn hash_array_7() {
    let mut hasher = Hasher::new();
    [0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0x40c40bfff532a623cbed6d12294d91c254cca2ef45f37aa2677314f4bea113ff);
    assert_eq(sha256([0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64]), 0x40c40bfff532a623cbed6d12294d91c254cca2ef45f37aa2677314f4bea113ff);
    assert_eq(hasher.keccak256(), 0x04298b70588eb138c0949dfd6ac76fd1f431f4cb8062ce0ac14215e3ce0b0bfb);
    assert_eq(keccak256([0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64]), 0x04298b70588eb138c0949dfd6ac76fd1f431f4cb8062ce0ac14215e3ce0b0bfb);

    let mut hasher = Hasher::new();
    [1_u64, 1_u64, 1u64, 1_u64, 1_u64, 1_u64, 1_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0x6c0373f769cb5bd7880f4693f9779685a8a35b1540924394916b5738bffb7e08);
    assert_eq(sha256([1_u64, 1_u64, 1u64, 1_u64, 1_u64, 1_u64, 1_u64]), 0x6c0373f769cb5bd7880f4693f9779685a8a35b1540924394916b5738bffb7e08);
    assert_eq(hasher.keccak256(), 0xbf21eb8a85e0dcb9c824378f2410b0199c76d197be9acb0175eadf5c4879df23);
    assert_eq(keccak256([1_u64, 1_u64, 1u64, 1_u64, 1_u64, 1_u64, 1_u64]), 0xbf21eb8a85e0dcb9c824378f2410b0199c76d197be9acb0175eadf5c4879df23);
}

#[cfg(experimental_new_hashing = false)]
#[test]
fn hash_array_8() {
    let mut hasher = Hasher::new();
    [0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0xf5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b);
    assert_eq(sha256([0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64]), 0xf5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b);
    assert_eq(hasher.keccak256(), 0xad3228b676f7d3cd4284a5443f17f1962b36e491b30a40b2405849e597ba5fb5);
    assert_eq(keccak256([0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64]), 0xad3228b676f7d3cd4284a5443f17f1962b36e491b30a40b2405849e597ba5fb5);

    let mut hasher = Hasher::new();
    [1_u64, 1_u64, 1u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0x794dde2d7e1d63dc28474122bd094bd35499447b3764dbf6cdf7c75ca73918dc);
    assert_eq(sha256([1_u64, 1_u64, 1u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64]), 0x794dde2d7e1d63dc28474122bd094bd35499447b3764dbf6cdf7c75ca73918dc);
    assert_eq(hasher.keccak256(), 0x4d7178fc9dd0659e4f25a41774e1905f280a2561076e876ee49350d24bdc9077);
    assert_eq(keccak256([1_u64, 1_u64, 1u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64]), 0x4d7178fc9dd0659e4f25a41774e1905f280a2561076e876ee49350d24bdc9077);
}

#[cfg(experimental_new_hashing = true)]
#[test]
fn hash_array_8() {
    let mut hasher = Hasher::new();
    [0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0xf709a410f5acbc14fe522dd5d0ccbd1dc50b8ad1dffdfbae6216b0eaa4857136);
    assert_eq(sha256([0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64]), 0xf709a410f5acbc14fe522dd5d0ccbd1dc50b8ad1dffdfbae6216b0eaa4857136);
    assert_eq(hasher.keccak256(), 0xeb692b9fb102487223961152496d31f6ee63f1c59b637f28b8f7601b64ba6dc4);
    assert_eq(keccak256([0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64]), 0xeb692b9fb102487223961152496d31f6ee63f1c59b637f28b8f7601b64ba6dc4);

    let mut hasher = Hasher::new();
    [1_u64, 1_u64, 1u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0xf170bfee84f4bc88f3ed0d7a17e1413cd24189d91b735f141188c5deddb512ab);
    assert_eq(sha256([1_u64, 1_u64, 1u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64]), 0xf170bfee84f4bc88f3ed0d7a17e1413cd24189d91b735f141188c5deddb512ab);
    assert_eq(hasher.keccak256(), 0x35fbd31a3699ba614c4d12f8ef07b32ebd644b8b8a2e592187bae4dd40432ad8);
    assert_eq(keccak256([1_u64, 1_u64, 1u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64]), 0x35fbd31a3699ba614c4d12f8ef07b32ebd644b8b8a2e592187bae4dd40432ad8);
}

#[cfg(experimental_new_hashing = false)]
#[test]
fn hash_array_9() {
    let mut hasher = Hasher::new();
    [0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0x834a709ba2534ebe3ee1397fd4f7bd288b2acc1d20a08d6c862dcd99b6f04400);
    assert_eq(sha256([0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64]), 0x834a709ba2534ebe3ee1397fd4f7bd288b2acc1d20a08d6c862dcd99b6f04400);
    assert_eq(hasher.keccak256(), 0x3cac317908c699fe873a7f6ee4e8cd63fbe9918b2315c97be91585590168e301);
    assert_eq(keccak256([0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64]), 0x3cac317908c699fe873a7f6ee4e8cd63fbe9918b2315c97be91585590168e301);

    let mut hasher = Hasher::new();
    [1_u64, 1_u64, 1u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0xe62386a1ec5b8fd0ece7344a7cae775d73179cfc0950c4fdeed26c7e8944e795);
    assert_eq(sha256([1_u64, 1_u64, 1u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64]), 0xe62386a1ec5b8fd0ece7344a7cae775d73179cfc0950c4fdeed26c7e8944e795);
    assert_eq(hasher.keccak256(), 0xf552e01b7c0e9b548702da13aeef5dafb7ae298fa02b0dd91dd24e89e1bb8462);
    assert_eq(keccak256([1_u64, 1_u64, 1u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64]), 0xf552e01b7c0e9b548702da13aeef5dafb7ae298fa02b0dd91dd24e89e1bb8462);
}

#[cfg(experimental_new_hashing = true)]
#[test]
fn hash_array_9() {
    let mut hasher = Hasher::new();
    [0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0x272eb8ab9a2300b562f7b9fda36893cc7a1fc1aaceb11256fe7af80628e5c678);
    assert_eq(sha256([0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64]), 0x272eb8ab9a2300b562f7b9fda36893cc7a1fc1aaceb11256fe7af80628e5c678);
    assert_eq(hasher.keccak256(), 0xdd5deb72d6c127312d3ff249276ea70c0bae1c4d0df12624fd1c40647830c324);
    assert_eq(keccak256([0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64]), 0xdd5deb72d6c127312d3ff249276ea70c0bae1c4d0df12624fd1c40647830c324);

    let mut hasher = Hasher::new();
    [1_u64, 1_u64, 1u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0x3901d66d0c73850aa34b48172ae9b53b8397e13878920cd26b58d3fa8e577e24);
    assert_eq(sha256([1_u64, 1_u64, 1u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64]), 0x3901d66d0c73850aa34b48172ae9b53b8397e13878920cd26b58d3fa8e577e24);
    assert_eq(hasher.keccak256(), 0x88a2aa30ec654cd24ead53507d631e4f4fc157cd99de2975ba65414da52a1bfb);
    assert_eq(keccak256([1_u64, 1_u64, 1u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64]), 0x88a2aa30ec654cd24ead53507d631e4f4fc157cd99de2975ba65414da52a1bfb);
}

#[cfg(experimental_new_hashing = false)]
#[test]
fn hash_array_10() {
    let mut hasher = Hasher::new();
    [0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0x5b6fb58e61fa475939767d68a446f97f1bff02c0e5935a3ea8bb51e6515783d8);
    assert_eq(sha256([0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64]), 0x5b6fb58e61fa475939767d68a446f97f1bff02c0e5935a3ea8bb51e6515783d8);
    assert_eq(hasher.keccak256(), 0x3a709301f7eafe917c7a06e209b077a9f3942799fb24b913407674a4c1485893);
    assert_eq(keccak256([0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64]), 0x3a709301f7eafe917c7a06e209b077a9f3942799fb24b913407674a4c1485893);

    let mut hasher = Hasher::new();
    [1_u64, 1_u64, 1u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0x5f80cf4c3ec64f652ea4ba4db7ea12896224546bd2ed4dd2032a8ce12fde16f9);
    assert_eq(sha256([1_u64, 1_u64, 1u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64]), 0x5f80cf4c3ec64f652ea4ba4db7ea12896224546bd2ed4dd2032a8ce12fde16f9);
    assert_eq(hasher.keccak256(), 0x48c5807e2d7a4a4d6568acae97a996fa10fbe2a664ffc97c86dbf883331962bd);
    assert_eq(keccak256([1_u64, 1_u64, 1u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64]), 0x48c5807e2d7a4a4d6568acae97a996fa10fbe2a664ffc97c86dbf883331962bd);
}

#[cfg(experimental_new_hashing = true)]
#[test]
fn hash_array_10() {
    let mut hasher = Hasher::new();
    [0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0xa1702307fb9aa2ada48d4f3c47b9be343a8a76361c492d1db68c2f11c2d6419c);
    assert_eq(sha256([0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64]), 0xa1702307fb9aa2ada48d4f3c47b9be343a8a76361c492d1db68c2f11c2d6419c);
    assert_eq(hasher.keccak256(), 0x76f005a71ff11b01cc2b8ddc40c8efab09f20c492d6ed221ef033e8ed3525172);
    assert_eq(keccak256([0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64]), 0x76f005a71ff11b01cc2b8ddc40c8efab09f20c492d6ed221ef033e8ed3525172);

    let mut hasher = Hasher::new();
    [1_u64, 1_u64, 1u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64].hash(hasher);
    assert_eq(hasher.sha256(), 0x5281f2dbb5b03aaf94b8db55b2f9ca63da69d650438087b499f03efa58809011);
    assert_eq(sha256([1_u64, 1_u64, 1u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64]), 0x5281f2dbb5b03aaf94b8db55b2f9ca63da69d650438087b499f03efa58809011);
    assert_eq(hasher.keccak256(), 0x76f00b7ebcc7b972bfcf7511a23c43b4047cc4fc2557fd651ee21813ddeaa014);
    assert_eq(keccak256([1_u64, 1_u64, 1u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64]), 0x76f00b7ebcc7b972bfcf7511a23c43b4047cc4fc2557fd651ee21813ddeaa014);
}

#[cfg(experimental_new_hashing = false)]
#[test]
fn hash_bytes() {
    let mut hasher = Hasher::new();
    let mut bytes = Bytes::new();
    bytes.hash(hasher);
    assert_eq(hasher.sha256(), 0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855);
    assert_eq(sha256(bytes), 0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855);
    assert_eq(hasher.keccak256(), 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470);
    assert_eq(keccak256(bytes), 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470);

    let mut hasher = Hasher::new();
    let mut bytes = Bytes::new();
    bytes.push(0_u8);
    bytes.hash(hasher);
    assert_eq(hasher.sha256(), 0x6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d);
    assert_eq(sha256(bytes), 0x6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d);
    assert_eq(hasher.keccak256(), 0xbc36789e7a1e281436464229828f817d6612f7b477d66591ff96a9e064bcc98a);
    assert_eq(keccak256(bytes), 0xbc36789e7a1e281436464229828f817d6612f7b477d66591ff96a9e064bcc98a);

    let mut hasher = Hasher::new();
    let mut bytes = Bytes::new();
    bytes.push(1_u8);
    bytes.hash(hasher);
    assert_eq(hasher.sha256(), 0x4bf5122f344554c53bde2ebb8cd2b7e3d1600ad631c385a5d7cce23c7785459a);
    assert_eq(sha256(bytes), 0x4bf5122f344554c53bde2ebb8cd2b7e3d1600ad631c385a5d7cce23c7785459a);
    assert_eq(hasher.keccak256(), 0x5fe7f977e71dba2ea1a68e21057beebb9be2ac30c6410aa38d4f3fbe41dcffd2);
    assert_eq(keccak256(bytes), 0x5fe7f977e71dba2ea1a68e21057beebb9be2ac30c6410aa38d4f3fbe41dcffd2);

    let mut bytes = Bytes::new();
    let mut i = 0;
    while i < 10 {
        bytes.push(0_u8);
        i += 1;
    }

    let mut hasher = Hasher::new();
    bytes.hash(hasher);
    assert_eq(hasher.sha256(), 0x01d448afd928065458cf670b60f5a594d735af0172c8d67f22a81680132681ca);
    assert_eq(sha256(bytes), 0x01d448afd928065458cf670b60f5a594d735af0172c8d67f22a81680132681ca);
    assert_eq(hasher.keccak256(), 0x6bd2dd6bd408cbee33429358bf24fdc64612fbf8b1b4db604518f40ffd34b607);
    assert_eq(keccak256(bytes), 0x6bd2dd6bd408cbee33429358bf24fdc64612fbf8b1b4db604518f40ffd34b607);

    let mut bytes = Bytes::new();
    let mut i = 0;
    while i < 10 {
        bytes.push(1_u8);
        i += 1;
    }

    let mut hasher = Hasher::new();
    bytes.hash(hasher);
    assert_eq(hasher.sha256(), 0xffadf8d89d37b3b55fe1847b513cf92e3be87e4c168708c7851845df96fb36be);
    assert_eq(sha256(bytes), 0xffadf8d89d37b3b55fe1847b513cf92e3be87e4c168708c7851845df96fb36be);
    assert_eq(hasher.keccak256(), 0xe3f42f79c06bc68dee65a965f26b9c1a1d40d3195f24341127150f7242979709);
    assert_eq(keccak256(bytes), 0xe3f42f79c06bc68dee65a965f26b9c1a1d40d3195f24341127150f7242979709);
}

#[cfg(experimental_new_hashing = true)]
#[test]
fn hash_bytes() {
    let mut hasher = Hasher::new();
    let mut bytes = Bytes::new();
    bytes.hash(hasher);
    assert_eq(hasher.sha256(), 0xaf5570f5a1810b7af78caf4bc70a660f0df51e42baf91d4de5b2328de0e83dfc);
    assert_eq(sha256(bytes), 0xaf5570f5a1810b7af78caf4bc70a660f0df51e42baf91d4de5b2328de0e83dfc);
    assert_eq(hasher.keccak256(), 0x011b4d03dd8c01f1049143cf9c4c817e4b167f1d1b83e5c6f0f10d89ba1e7bce);
    assert_eq(keccak256(bytes), 0x011b4d03dd8c01f1049143cf9c4c817e4b167f1d1b83e5c6f0f10d89ba1e7bce);

    let mut hasher = Hasher::new();
    let mut bytes = Bytes::new();
    bytes.push(0_u8);
    bytes.hash(hasher);
    assert_eq(hasher.sha256(), 0xe64cf59bfbcf3c5743ccd9eda3a811a7966689717a8499e5b1ef0cb1f33bf4b6);
    assert_eq(sha256(bytes), 0xe64cf59bfbcf3c5743ccd9eda3a811a7966689717a8499e5b1ef0cb1f33bf4b6);
    assert_eq(hasher.keccak256(), 0x4ca50a38e76ab659f435e61e6b5aaf81ce1b52eb5330ec73caddf64f28162253);
    assert_eq(keccak256(bytes), 0x4ca50a38e76ab659f435e61e6b5aaf81ce1b52eb5330ec73caddf64f28162253);

    let mut hasher = Hasher::new();
    let mut bytes = Bytes::new();
    bytes.push(1_u8);
    bytes.hash(hasher);
    assert_eq(hasher.sha256(), 0xbd87b2cda99df5b642ac9c0a97d3bc76f9921e2cce16058faa44bc954dbb065f);
    assert_eq(sha256(bytes), 0xbd87b2cda99df5b642ac9c0a97d3bc76f9921e2cce16058faa44bc954dbb065f);
    assert_eq(hasher.keccak256(), 0x37dc9801eac37f6c32e8f1643e6d8d447bb68412a514dc85c094dbca25026b9f);
    assert_eq(keccak256(bytes), 0x37dc9801eac37f6c32e8f1643e6d8d447bb68412a514dc85c094dbca25026b9f);

    let mut bytes = Bytes::new();
    let mut i = 0;
    while i < 10 {
        bytes.push(0_u8);
        i += 1;
    }

    let mut hasher = Hasher::new();
    bytes.hash(hasher);
    assert_eq(hasher.sha256(), 0x30b823d93f132f0511517e9797e5608d7a2fedfee3aa352b969d815e6fb97f6a);
    assert_eq(sha256(bytes), 0x30b823d93f132f0511517e9797e5608d7a2fedfee3aa352b969d815e6fb97f6a);
    assert_eq(hasher.keccak256(), 0x77c5192c37c8baf3f8eba33cc4d48ed38ae27fbe3e90f06c0042ae6078fb5674);
    assert_eq(keccak256(bytes), 0x77c5192c37c8baf3f8eba33cc4d48ed38ae27fbe3e90f06c0042ae6078fb5674);

    let mut bytes = Bytes::new();
    let mut i = 0;
    while i < 10 {
        bytes.push(1_u8);
        i += 1;
    }

    let mut hasher = Hasher::new();
    bytes.hash(hasher);
    assert_eq(hasher.sha256(), 0x1e0e565120a6ceaf0797ad7e6600ed1edce68aa86ef83e1057e3225edc382957);
    assert_eq(sha256(bytes), 0x1e0e565120a6ceaf0797ad7e6600ed1edce68aa86ef83e1057e3225edc382957);
    assert_eq(hasher.keccak256(), 0x56dc7330cf2017962f405489806ca1b7fc537f382fbe0ac995d0dd947e80f72a);
    assert_eq(keccak256(bytes), 0x56dc7330cf2017962f405489806ca1b7fc537f382fbe0ac995d0dd947e80f72a);
}

#[cfg(experimental_new_hashing = false)]
#[test]
fn hash_vec() {
    let vec = Vec::<u64>::new();
    let mut hasher = Hasher::new();
    vec.hash(hasher);
    assert_eq(hasher.sha256(), 0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855);
    assert_eq(sha256(vec), 0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855);
    assert_eq(hasher.keccak256(), 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470);
    assert_eq(keccak256(vec), 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470);

    let mut vec = Vec::<u64>::new();
    let mut i = 0;
    while i < 10 {
        vec.push(0_u64);
        i += 1;
    }

    let mut hasher = Hasher::new();
    vec.hash(hasher);
    assert_eq(hasher.sha256(), 0x5b6fb58e61fa475939767d68a446f97f1bff02c0e5935a3ea8bb51e6515783d8);
    assert_eq(sha256(vec), 0x5b6fb58e61fa475939767d68a446f97f1bff02c0e5935a3ea8bb51e6515783d8);
    assert_eq(hasher.keccak256(), 0x3a709301f7eafe917c7a06e209b077a9f3942799fb24b913407674a4c1485893);
    assert_eq(keccak256(vec), 0x3a709301f7eafe917c7a06e209b077a9f3942799fb24b913407674a4c1485893);

    let mut vec = Vec::<u64>::new();
    let mut i = 0;
    while i < 10 {
        vec.push(1_u64);
        i += 1;
    }

    let mut hasher = Hasher::new();
    vec.hash(hasher);
    assert_eq(hasher.sha256(), 0x5f80cf4c3ec64f652ea4ba4db7ea12896224546bd2ed4dd2032a8ce12fde16f9);
    assert_eq(sha256(vec), 0x5f80cf4c3ec64f652ea4ba4db7ea12896224546bd2ed4dd2032a8ce12fde16f9);
    assert_eq(hasher.keccak256(), 0x48c5807e2d7a4a4d6568acae97a996fa10fbe2a664ffc97c86dbf883331962bd);
    assert_eq(keccak256(vec), 0x48c5807e2d7a4a4d6568acae97a996fa10fbe2a664ffc97c86dbf883331962bd);

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
    assert_eq(hasher.sha256(), 0x5b6fb58e61fa475939767d68a446f97f1bff02c0e5935a3ea8bb51e6515783d8);
    assert_eq(sha256(vec), 0x5b6fb58e61fa475939767d68a446f97f1bff02c0e5935a3ea8bb51e6515783d8);
    assert_eq(hasher.keccak256(), 0x3a709301f7eafe917c7a06e209b077a9f3942799fb24b913407674a4c1485893);
    assert_eq(keccak256(vec), 0x3a709301f7eafe917c7a06e209b077a9f3942799fb24b913407674a4c1485893);
}

#[cfg(experimental_new_hashing = true)]
#[test]
fn hash_vec() {
    let vec = Vec::<u64>::new();
    let mut hasher = Hasher::new();
    vec.hash(hasher);
    assert_eq(hasher.sha256(), 0xaf5570f5a1810b7af78caf4bc70a660f0df51e42baf91d4de5b2328de0e83dfc);
    assert_eq(sha256(vec), 0xaf5570f5a1810b7af78caf4bc70a660f0df51e42baf91d4de5b2328de0e83dfc);
    assert_eq(hasher.keccak256(), 0x011b4d03dd8c01f1049143cf9c4c817e4b167f1d1b83e5c6f0f10d89ba1e7bce);
    assert_eq(keccak256(vec), 0x011b4d03dd8c01f1049143cf9c4c817e4b167f1d1b83e5c6f0f10d89ba1e7bce);

    let mut vec = Vec::<u64>::new();
    let mut i = 0;
    while i < 10 {
        vec.push(0_u64);
        i += 1;
    }

    let mut hasher = Hasher::new();
    vec.hash(hasher);
    assert_eq(hasher.sha256(), 0xa1702307fb9aa2ada48d4f3c47b9be343a8a76361c492d1db68c2f11c2d6419c);
    assert_eq(sha256(vec), 0xa1702307fb9aa2ada48d4f3c47b9be343a8a76361c492d1db68c2f11c2d6419c);
    assert_eq(hasher.keccak256(), 0x76f005a71ff11b01cc2b8ddc40c8efab09f20c492d6ed221ef033e8ed3525172);
    assert_eq(keccak256(vec), 0x76f005a71ff11b01cc2b8ddc40c8efab09f20c492d6ed221ef033e8ed3525172);

    let mut vec = Vec::<u64>::new();
    let mut i = 0;
    while i < 10 {
        vec.push(1_u64);
        i += 1;
    }

    let mut hasher = Hasher::new();
    vec.hash(hasher);
    assert_eq(hasher.sha256(), 0x5281f2dbb5b03aaf94b8db55b2f9ca63da69d650438087b499f03efa58809011);
    assert_eq(sha256(vec), 0x5281f2dbb5b03aaf94b8db55b2f9ca63da69d650438087b499f03efa58809011);
    assert_eq(hasher.keccak256(), 0x76f00b7ebcc7b972bfcf7511a23c43b4047cc4fc2557fd651ee21813ddeaa014);
    assert_eq(keccak256(vec), 0x76f00b7ebcc7b972bfcf7511a23c43b4047cc4fc2557fd651ee21813ddeaa014);

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
    assert_eq(hasher.sha256(), 0xf5e198ceca6aa3f49297ca70adda28a6258e45fc82e3c6a67482097c1e9ab576);
    assert_eq(sha256(vec), 0xf5e198ceca6aa3f49297ca70adda28a6258e45fc82e3c6a67482097c1e9ab576);
    assert_eq(hasher.keccak256(), 0x4ad52f2573a008ff4024efde1f86fe5049392064c7dcbe22fa617a97573ebfbd);
    assert_eq(keccak256(vec), 0x4ad52f2573a008ff4024efde1f86fe5049392064c7dcbe22fa617a97573ebfbd);
}

#[test()]
fn hash_address() {
    let mut hasher = Hasher::new();
    let address = Address::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    address.hash(hasher);
    assert_eq(hasher.sha256(), 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);
    assert_eq(sha256(address), 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);
    assert_eq(hasher.keccak256(), 0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563);
    assert_eq(keccak256(address), 0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563);

    let mut hasher = Hasher::new();
    let address = Address::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    address.hash(hasher);
    assert_eq(hasher.sha256(), 0xec4916dd28fc4c10d78e287ca5d9cc51ee1ae73cbfde08c6b37324cbfaac8bc5);
    assert_eq(sha256(address), 0xec4916dd28fc4c10d78e287ca5d9cc51ee1ae73cbfde08c6b37324cbfaac8bc5);
    assert_eq(hasher.keccak256(), 0xb10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6);
    assert_eq(keccak256(address), 0xb10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6);

    let mut hasher = Hasher::new();
    let address = Address::from(0x000000000000000000000000000000000000000000000000000000000000002a);
    address.hash(hasher);
    assert_eq(hasher.sha256(), 0x0a28e9ffef0073f9a6a674cf57ee77307f38f0f1bebb087888d9011ed0eeefdf);
    assert_eq(sha256(address), 0x0a28e9ffef0073f9a6a674cf57ee77307f38f0f1bebb087888d9011ed0eeefdf);
    assert_eq(hasher.keccak256(), 0xbeced09521047d05b8960b7e7bcc1d1292cf3e4b2a6b63f48335cbde5f7545d2);
    assert_eq(keccak256(address), 0xbeced09521047d05b8960b7e7bcc1d1292cf3e4b2a6b63f48335cbde5f7545d2);

    let mut hasher = Hasher::new();
    let address = Address::from(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff);
    address.hash(hasher);
    assert_eq(hasher.sha256(), 0xaf9613760f72635fbdb44a5a0a63c39f12af30f950a6ee5c971be188e89c4051);
    assert_eq(sha256(address), 0xaf9613760f72635fbdb44a5a0a63c39f12af30f950a6ee5c971be188e89c4051);
    assert_eq(hasher.keccak256(), 0xa9c584056064687e149968cbab758a3376d22aedc6a55823d1b3ecbee81b8fb9);
    assert_eq(keccak256(address), 0xa9c584056064687e149968cbab758a3376d22aedc6a55823d1b3ecbee81b8fb9);
}

#[test()]
fn hash_asset_id() {
    let mut hasher = Hasher::new();
    let asset_id = AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    asset_id.hash(hasher);
    assert_eq(hasher.sha256(), 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);
    assert_eq(sha256(asset_id), 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);
    assert_eq(hasher.keccak256(), 0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563);
    assert_eq(keccak256(asset_id), 0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563);

    let mut hasher = Hasher::new();
    let asset_id = AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    asset_id.hash(hasher);
    assert_eq(hasher.sha256(), 0xec4916dd28fc4c10d78e287ca5d9cc51ee1ae73cbfde08c6b37324cbfaac8bc5);
    assert_eq(sha256(asset_id), 0xec4916dd28fc4c10d78e287ca5d9cc51ee1ae73cbfde08c6b37324cbfaac8bc5);
    assert_eq(hasher.keccak256(), 0xb10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6);
    assert_eq(keccak256(asset_id), 0xb10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6);

    let mut hasher = Hasher::new();
    let asset_id = AssetId::from(0x000000000000000000000000000000000000000000000000000000000000002a);
    asset_id.hash(hasher);
    assert_eq(hasher.sha256(), 0x0a28e9ffef0073f9a6a674cf57ee77307f38f0f1bebb087888d9011ed0eeefdf);
    assert_eq(sha256(asset_id), 0x0a28e9ffef0073f9a6a674cf57ee77307f38f0f1bebb087888d9011ed0eeefdf);
    assert_eq(hasher.keccak256(), 0xbeced09521047d05b8960b7e7bcc1d1292cf3e4b2a6b63f48335cbde5f7545d2);
    assert_eq(keccak256(asset_id), 0xbeced09521047d05b8960b7e7bcc1d1292cf3e4b2a6b63f48335cbde5f7545d2);

    let mut hasher = Hasher::new();
    let asset_id = AssetId::from(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff);
    asset_id.hash(hasher);
    assert_eq(hasher.sha256(), 0xaf9613760f72635fbdb44a5a0a63c39f12af30f950a6ee5c971be188e89c4051);
    assert_eq(sha256(asset_id), 0xaf9613760f72635fbdb44a5a0a63c39f12af30f950a6ee5c971be188e89c4051);
    assert_eq(hasher.keccak256(), 0xa9c584056064687e149968cbab758a3376d22aedc6a55823d1b3ecbee81b8fb9);
    assert_eq(keccak256(asset_id), 0xa9c584056064687e149968cbab758a3376d22aedc6a55823d1b3ecbee81b8fb9);
}

#[test()]
fn hash_contract_id() {
    let mut hasher = Hasher::new();
    let contract_id = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    contract_id.hash(hasher);
    assert_eq(hasher.sha256(), 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);
    assert_eq(sha256(contract_id), 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);
    assert_eq(hasher.keccak256(), 0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563);
    assert_eq(keccak256(contract_id), 0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563);

    let mut hasher = Hasher::new();
    let contract_id = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    contract_id.hash(hasher);
    assert_eq(hasher.sha256(), 0xec4916dd28fc4c10d78e287ca5d9cc51ee1ae73cbfde08c6b37324cbfaac8bc5);
    assert_eq(sha256(contract_id), 0xec4916dd28fc4c10d78e287ca5d9cc51ee1ae73cbfde08c6b37324cbfaac8bc5);
    assert_eq(hasher.keccak256(), 0xb10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6);
    assert_eq(keccak256(contract_id), 0xb10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6);

    let mut hasher = Hasher::new();
    let contract_id = ContractId::from(0x000000000000000000000000000000000000000000000000000000000000002a);
    contract_id.hash(hasher);
    assert_eq(hasher.sha256(), 0x0a28e9ffef0073f9a6a674cf57ee77307f38f0f1bebb087888d9011ed0eeefdf);
    assert_eq(sha256(contract_id), 0x0a28e9ffef0073f9a6a674cf57ee77307f38f0f1bebb087888d9011ed0eeefdf);
    assert_eq(hasher.keccak256(), 0xbeced09521047d05b8960b7e7bcc1d1292cf3e4b2a6b63f48335cbde5f7545d2);
    assert_eq(keccak256(contract_id), 0xbeced09521047d05b8960b7e7bcc1d1292cf3e4b2a6b63f48335cbde5f7545d2);

    let mut hasher = Hasher::new();
    let contract_id = ContractId::from(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff);
    contract_id.hash(hasher);
    assert_eq(hasher.sha256(), 0xaf9613760f72635fbdb44a5a0a63c39f12af30f950a6ee5c971be188e89c4051);
    assert_eq(sha256(contract_id), 0xaf9613760f72635fbdb44a5a0a63c39f12af30f950a6ee5c971be188e89c4051);
    assert_eq(hasher.keccak256(), 0xa9c584056064687e149968cbab758a3376d22aedc6a55823d1b3ecbee81b8fb9);
    assert_eq(keccak256(contract_id), 0xa9c584056064687e149968cbab758a3376d22aedc6a55823d1b3ecbee81b8fb9);
}

#[test()]
fn hash_identity() {
    // Identity::Address variant.
    let mut hasher = Hasher::new();
    let identity = Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000000));
    identity.hash(hasher);
    assert_eq(hasher.sha256(), 0x7f9c9e31ac8256ca2f258583df262dbc7d6f68f2a03043d5c99a4ae5a7396ce9);
    assert_eq(sha256(identity), 0x7f9c9e31ac8256ca2f258583df262dbc7d6f68f2a03043d5c99a4ae5a7396ce9);
    assert_eq(hasher.keccak256(), 0xf39a869f62e75cf5f0bf914688a6b289caf2049435d8e68c5c5e6d05e44913f3);
    assert_eq(keccak256(identity), 0xf39a869f62e75cf5f0bf914688a6b289caf2049435d8e68c5c5e6d05e44913f3);

    let mut hasher = Hasher::new();
    let identity = Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    identity.hash(hasher);
    assert_eq(hasher.sha256(), 0x1fd4247443c9440cb3c48c28851937196bc156032d70a96c98e127ecb347e45f);
    assert_eq(sha256(identity), 0x1fd4247443c9440cb3c48c28851937196bc156032d70a96c98e127ecb347e45f);
    assert_eq(hasher.keccak256(), 0xc13ad76448cbefd1ee83b801bcd8f33061f2577d6118395e7b44ea21c7ef62e0);
    assert_eq(keccak256(identity), 0xc13ad76448cbefd1ee83b801bcd8f33061f2577d6118395e7b44ea21c7ef62e0);

    let mut hasher = Hasher::new();
    let identity = Identity::Address(Address::from(0x000000000000000000000000000000000000000000000000000000000000002a));
    identity.hash(hasher);
    assert_eq(hasher.sha256(), 0xeda2bb11a7b275bd1ca710aef2e01d5d245b76b8966ef8dbac5935e637ad3a69);
    assert_eq(sha256(identity), 0xeda2bb11a7b275bd1ca710aef2e01d5d245b76b8966ef8dbac5935e637ad3a69);
    assert_eq(hasher.keccak256(), 0x2bdb76988595837f708230562ad1a4f4efb3d358e4c778478dcaae229c64a0fd);
    assert_eq(keccak256(identity), 0x2bdb76988595837f708230562ad1a4f4efb3d358e4c778478dcaae229c64a0fd);

    let mut hasher = Hasher::new();
    let identity = Identity::Address(Address::from(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff));
    identity.hash(hasher);
    assert_eq(hasher.sha256(), 0x5e16d316ecd5773e50c3b02737d424192b02f25b4245822079181c557aafda7d);
    assert_eq(sha256(identity), 0x5e16d316ecd5773e50c3b02737d424192b02f25b4245822079181c557aafda7d);
    assert_eq(hasher.keccak256(), 0x831fe840469ac85581e3a78ca61980fecd4dfe720ab051fe5300153b77f28e4f);
    assert_eq(keccak256(identity), 0x831fe840469ac85581e3a78ca61980fecd4dfe720ab051fe5300153b77f28e4f);

    // Identity::ContractId variant.
    let mut hasher = Hasher::new();
    let identity = Identity::ContractId(ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000));
    identity.hash(hasher);
    assert_eq(hasher.sha256(), 0x1a7dfdeaffeedac489287e85be5e9c049a2ff6470f55cf30260f55395ac1b159);
    assert_eq(sha256(identity), 0x1a7dfdeaffeedac489287e85be5e9c049a2ff6470f55cf30260f55395ac1b159);
    assert_eq(hasher.keccak256(), 0x0d678e31a4b2825b806fe160675cd01dab159802c7f94397ce45ed91b5f3aac6);
    assert_eq(keccak256(identity), 0x0d678e31a4b2825b806fe160675cd01dab159802c7f94397ce45ed91b5f3aac6);

    let mut hasher = Hasher::new();
    let identity = Identity::ContractId(ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    identity.hash(hasher);
    assert_eq(hasher.sha256(), 0x2e255099d6d6bee307c8e7075acc78f949897c5f67b53adf60724c814d7b90cb);
    assert_eq(sha256(identity), 0x2e255099d6d6bee307c8e7075acc78f949897c5f67b53adf60724c814d7b90cb);
    assert_eq(hasher.keccak256(), 0x9b68e489a07c86105b2c34adda59d3851d6f33abd41be6e9559cf783147db5dd);
    assert_eq(keccak256(identity), 0x9b68e489a07c86105b2c34adda59d3851d6f33abd41be6e9559cf783147db5dd);

    let mut hasher = Hasher::new();
    let identity = Identity::ContractId(ContractId::from(0x000000000000000000000000000000000000000000000000000000000000002a));
    identity.hash(hasher);
    assert_eq(hasher.sha256(), 0xc69bfa18830e0f03230520a6d504b3a958d395214560575317d524d8731dfb58);
    assert_eq(sha256(identity), 0xc69bfa18830e0f03230520a6d504b3a958d395214560575317d524d8731dfb58);
    assert_eq(hasher.keccak256(), 0x109c7d1a56a8d4555ebed5c963048374daedb9b1e99458bd3683101437843e0e);
    assert_eq(keccak256(identity), 0x109c7d1a56a8d4555ebed5c963048374daedb9b1e99458bd3683101437843e0e);

    let mut hasher = Hasher::new();
    let identity = Identity::ContractId(ContractId::from(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff));
    identity.hash(hasher);
    assert_eq(hasher.sha256(), 0x29fb7cd3be48a8d76bb031f0abce26caa9e092c000cd16bb101d30f63c4c1bc1);
    assert_eq(sha256(identity), 0x29fb7cd3be48a8d76bb031f0abce26caa9e092c000cd16bb101d30f63c4c1bc1);
    assert_eq(hasher.keccak256(), 0x02a0f2d2b92e1b8c9cc103818a30b344401fce2fb233101610dfdd1a24afee09);
    assert_eq(keccak256(identity), 0x02a0f2d2b92e1b8c9cc103818a30b344401fce2fb233101610dfdd1a24afee09);
}

#[cfg(experimental_new_hashing = false)]
#[test()]
fn hash_string() {
    let mut hasher = Hasher::new();
    let string = String::from("");
    string.hash(hasher);
    assert_eq(hasher.sha256(), 0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855);
    assert_eq(sha256(string), 0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855);
    assert_eq(hasher.keccak256(), 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470);
    assert_eq(keccak256(string), 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470);

    let mut hasher = Hasher::new();
    let string = String::from("test");
    string.hash(hasher);
    assert_eq(hasher.sha256(), 0x9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08);
    assert_eq(sha256(string), 0x9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08);
    assert_eq(hasher.keccak256(), 0x9c22ff5f21f0b81b113e63f7db6da94fedef11b2119b4088b89664fb9a3cb658);
    assert_eq(keccak256(string), 0x9c22ff5f21f0b81b113e63f7db6da94fedef11b2119b4088b89664fb9a3cb658);

    let mut hasher = Hasher::new();
    let string = String::from("Fastest Modular Execution Layer!");
    string.hash(hasher);
    assert_eq(hasher.sha256(), 0x4a3cd7c8b44dbf7941e55179425f746adeaa97fe2d99b571fffee78e9b41743c);
    assert_eq(sha256(string), 0x4a3cd7c8b44dbf7941e55179425f746adeaa97fe2d99b571fffee78e9b41743c);
    assert_eq(hasher.keccak256(), 0xab8e83e041e001bcf797c9cc7d6bc472bfdb8c736bab7999f13b7c26f48c354f);
    assert_eq(keccak256(string), 0xab8e83e041e001bcf797c9cc7d6bc472bfdb8c736bab7999f13b7c26f48c354f);
}

#[cfg(experimental_new_hashing = true)]
#[test()]
fn hash_string() {
    let mut hasher = Hasher::new();
    let string = String::from("");
    string.hash(hasher);
    assert_eq(hasher.sha256(), 0xaf5570f5a1810b7af78caf4bc70a660f0df51e42baf91d4de5b2328de0e83dfc);
    assert_eq(sha256(string), 0xaf5570f5a1810b7af78caf4bc70a660f0df51e42baf91d4de5b2328de0e83dfc);
    assert_eq(hasher.keccak256(), 0x011b4d03dd8c01f1049143cf9c4c817e4b167f1d1b83e5c6f0f10d89ba1e7bce);
    assert_eq(keccak256(string), 0x011b4d03dd8c01f1049143cf9c4c817e4b167f1d1b83e5c6f0f10d89ba1e7bce);

    let mut hasher = Hasher::new();
    let string = String::from("test");
    string.hash(hasher);
    assert_eq(hasher.sha256(), 0x09a7d352412717c7e0b93286eb544f83ddf6da4260b795e90aa44e8e58f5dadd);
    assert_eq(sha256(string), 0x09a7d352412717c7e0b93286eb544f83ddf6da4260b795e90aa44e8e58f5dadd);
    assert_eq(hasher.keccak256(), 0x7deeee38ddc74b84935b679921e2554392d9228f46f9845e4f379a3a67635ccd);
    assert_eq(keccak256(string), 0x7deeee38ddc74b84935b679921e2554392d9228f46f9845e4f379a3a67635ccd);

    let mut hasher = Hasher::new();
    let string = String::from("Fastest Modular Execution Layer!");
    string.hash(hasher);
    assert_eq(hasher.sha256(), 0x03e88f60c46971ad474fbcc4b8532136a378b140f5eeb2b26cb490dbd10c51e8);
    assert_eq(sha256(string), 0x03e88f60c46971ad474fbcc4b8532136a378b140f5eeb2b26cb490dbd10c51e8);
    assert_eq(hasher.keccak256(), 0x61196ca4771dd4c6c645c2f9f14c7a45c64247b05eb81dd9ffd7ebc68b5b2f7c);
    assert_eq(keccak256(string), 0x61196ca4771dd4c6c645c2f9f14c7a45c64247b05eb81dd9ffd7ebc68b5b2f7c);
}

#[test()]
fn hash_ed25519() {
    let mut hasher = Hasher::new();
    let ed25519 = Ed25519::from((b256::min(), 0x0000000000000000000000000000000000000000000000000000000000000000));
    ed25519.hash(hasher);
    assert_eq(hasher.sha256(), 0xf5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b);
    assert_eq(sha256(ed25519), 0xf5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b);
    assert_eq(hasher.keccak256(), 0xad3228b676f7d3cd4284a5443f17f1962b36e491b30a40b2405849e597ba5fb5);
    assert_eq(keccak256(ed25519), 0xad3228b676f7d3cd4284a5443f17f1962b36e491b30a40b2405849e597ba5fb5);

    let mut hasher = Hasher::new();
    let ed25519 = Ed25519::from((b256::min(), 0x0000000000000000000000000000000000000000000000000000000000000001));
    ed25519.hash(hasher);
    assert_eq(hasher.sha256(), 0x90f4b39548df55ad6187a1d20d731ecee78c545b94afd16f42ef7592d99cd365);
    assert_eq(sha256(ed25519), 0x90f4b39548df55ad6187a1d20d731ecee78c545b94afd16f42ef7592d99cd365);
    assert_eq(hasher.keccak256(), 0xa6eef7e35abe7026729641147f7915573c7e97b47efa546f5f6e3230263bcb49);
    assert_eq(keccak256(ed25519), 0xa6eef7e35abe7026729641147f7915573c7e97b47efa546f5f6e3230263bcb49);

    let mut hasher = Hasher::new();
    let ed25519 = Ed25519::from((b256::min(), 0x000000000000000000000000000000000000000000000000000000000000002a));
    ed25519.hash(hasher);
    assert_eq(hasher.sha256(), 0xc77673a8cc11eb4f660ce1a4ca446423df3b68677ba4c1c1846351ddbeb2e5ef);
    assert_eq(sha256(ed25519), 0xc77673a8cc11eb4f660ce1a4ca446423df3b68677ba4c1c1846351ddbeb2e5ef);
    assert_eq(hasher.keccak256(), 0x25a1a901705ed15d5376e82511cff743d9474883c82d145cebcc7811e0424a9c);
    assert_eq(keccak256(ed25519), 0x25a1a901705ed15d5376e82511cff743d9474883c82d145cebcc7811e0424a9c);

    let mut hasher = Hasher::new();
    let ed25519 = Ed25519::from((b256::max(), 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff));
    ed25519.hash(hasher);
    assert_eq(hasher.sha256(), 0x8667e718294e9e0df1d30600ba3eeb201f764aad2dad72748643e4a285e1d1f7);
    assert_eq(sha256(ed25519), 0x8667e718294e9e0df1d30600ba3eeb201f764aad2dad72748643e4a285e1d1f7);
    assert_eq(hasher.keccak256(), 0xbd8b151773dbbefd7b0df67f2dcc482901728b6df477f4fb2f192733a005d396);
    assert_eq(keccak256(ed25519), 0xbd8b151773dbbefd7b0df67f2dcc482901728b6df477f4fb2f192733a005d396);
}

#[test()]
fn hash_message() {
    let mut hasher = Hasher::new();
    let message = Message::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    message.hash(hasher);
    assert_eq(hasher.sha256(), 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);
    assert_eq(sha256(message), 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);
    assert_eq(hasher.keccak256(), 0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563);
    assert_eq(keccak256(message), 0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563);

    let mut hasher = Hasher::new();
    let message = Message::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    message.hash(hasher);
    assert_eq(hasher.sha256(), 0xec4916dd28fc4c10d78e287ca5d9cc51ee1ae73cbfde08c6b37324cbfaac8bc5);
    assert_eq(sha256(message), 0xec4916dd28fc4c10d78e287ca5d9cc51ee1ae73cbfde08c6b37324cbfaac8bc5);
    assert_eq(hasher.keccak256(), 0xb10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6);
    assert_eq(keccak256(message), 0xb10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6);

    let mut hasher = Hasher::new();
    let message = Message::from(0x000000000000000000000000000000000000000000000000000000000000002a);
    message.hash(hasher);
    assert_eq(hasher.sha256(), 0x0a28e9ffef0073f9a6a674cf57ee77307f38f0f1bebb087888d9011ed0eeefdf);
    assert_eq(sha256(message), 0x0a28e9ffef0073f9a6a674cf57ee77307f38f0f1bebb087888d9011ed0eeefdf);
    assert_eq(hasher.keccak256(), 0xbeced09521047d05b8960b7e7bcc1d1292cf3e4b2a6b63f48335cbde5f7545d2);
    assert_eq(keccak256(message), 0xbeced09521047d05b8960b7e7bcc1d1292cf3e4b2a6b63f48335cbde5f7545d2);

    let mut hasher = Hasher::new();
    let message = Message::from(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff);
    message.hash(hasher);
    assert_eq(hasher.sha256(), 0xaf9613760f72635fbdb44a5a0a63c39f12af30f950a6ee5c971be188e89c4051);
    assert_eq(sha256(message), 0xaf9613760f72635fbdb44a5a0a63c39f12af30f950a6ee5c971be188e89c4051);
    assert_eq(hasher.keccak256(), 0xa9c584056064687e149968cbab758a3376d22aedc6a55823d1b3ecbee81b8fb9);
    assert_eq(keccak256(message), 0xa9c584056064687e149968cbab758a3376d22aedc6a55823d1b3ecbee81b8fb9);
}

#[test()]
fn hash_public_key() {
    let mut hasher = Hasher::new();
    let public_key = PublicKey::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    public_key.hash(hasher);
    assert_eq(hasher.sha256(), 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);
    assert_eq(sha256(public_key), 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);
    assert_eq(hasher.keccak256(), 0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563);
    assert_eq(keccak256(public_key), 0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563);

    let mut hasher = Hasher::new();
    let public_key = PublicKey::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    public_key.hash(hasher);
    assert_eq(hasher.sha256(), 0xec4916dd28fc4c10d78e287ca5d9cc51ee1ae73cbfde08c6b37324cbfaac8bc5);
    assert_eq(sha256(public_key), 0xec4916dd28fc4c10d78e287ca5d9cc51ee1ae73cbfde08c6b37324cbfaac8bc5);
    assert_eq(hasher.keccak256(), 0xb10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6);
    assert_eq(keccak256(public_key), 0xb10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6);

    let mut hasher = Hasher::new();
    let public_key = PublicKey::from(0x000000000000000000000000000000000000000000000000000000000000002a);
    public_key.hash(hasher);
    assert_eq(hasher.sha256(), 0x0a28e9ffef0073f9a6a674cf57ee77307f38f0f1bebb087888d9011ed0eeefdf);
    assert_eq(sha256(public_key), 0x0a28e9ffef0073f9a6a674cf57ee77307f38f0f1bebb087888d9011ed0eeefdf);
    assert_eq(hasher.keccak256(), 0xbeced09521047d05b8960b7e7bcc1d1292cf3e4b2a6b63f48335cbde5f7545d2);
    assert_eq(keccak256(public_key), 0xbeced09521047d05b8960b7e7bcc1d1292cf3e4b2a6b63f48335cbde5f7545d2);

    let mut hasher = Hasher::new();
    let public_key = PublicKey::from(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff);
    public_key.hash(hasher);
    assert_eq(hasher.sha256(), 0xaf9613760f72635fbdb44a5a0a63c39f12af30f950a6ee5c971be188e89c4051);
    assert_eq(sha256(public_key), 0xaf9613760f72635fbdb44a5a0a63c39f12af30f950a6ee5c971be188e89c4051);
    assert_eq(hasher.keccak256(), 0xa9c584056064687e149968cbab758a3376d22aedc6a55823d1b3ecbee81b8fb9);
    assert_eq(keccak256(public_key), 0xa9c584056064687e149968cbab758a3376d22aedc6a55823d1b3ecbee81b8fb9);
}

#[test()]
fn hash_secp256k1() {
    let mut hasher = Hasher::new();
    let secp256k1 = Secp256k1::from((b256::min(), 0x0000000000000000000000000000000000000000000000000000000000000000));
    secp256k1.hash(hasher);
    assert_eq(hasher.sha256(), 0xf5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b);
    assert_eq(sha256(secp256k1), 0xf5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b);
    assert_eq(hasher.keccak256(), 0xad3228b676f7d3cd4284a5443f17f1962b36e491b30a40b2405849e597ba5fb5);
    assert_eq(keccak256(secp256k1), 0xad3228b676f7d3cd4284a5443f17f1962b36e491b30a40b2405849e597ba5fb5);

    let mut hasher = Hasher::new();
    let secp256k1 = Secp256k1::from((b256::min(), 0x0000000000000000000000000000000000000000000000000000000000000001));
    secp256k1.hash(hasher);
    assert_eq(hasher.sha256(), 0x90f4b39548df55ad6187a1d20d731ecee78c545b94afd16f42ef7592d99cd365);
    assert_eq(sha256(secp256k1), 0x90f4b39548df55ad6187a1d20d731ecee78c545b94afd16f42ef7592d99cd365);
    assert_eq(hasher.keccak256(), 0xa6eef7e35abe7026729641147f7915573c7e97b47efa546f5f6e3230263bcb49);
    assert_eq(keccak256(secp256k1), 0xa6eef7e35abe7026729641147f7915573c7e97b47efa546f5f6e3230263bcb49);

    let mut hasher = Hasher::new();
    let secp256k1 = Secp256k1::from((b256::min(), 0x000000000000000000000000000000000000000000000000000000000000002a));
    secp256k1.hash(hasher);
    assert_eq(hasher.sha256(), 0xc77673a8cc11eb4f660ce1a4ca446423df3b68677ba4c1c1846351ddbeb2e5ef);
    assert_eq(sha256(secp256k1), 0xc77673a8cc11eb4f660ce1a4ca446423df3b68677ba4c1c1846351ddbeb2e5ef);
    assert_eq(hasher.keccak256(), 0x25a1a901705ed15d5376e82511cff743d9474883c82d145cebcc7811e0424a9c);
    assert_eq(keccak256(secp256k1), 0x25a1a901705ed15d5376e82511cff743d9474883c82d145cebcc7811e0424a9c);

    let mut hasher = Hasher::new();
    let secp256k1 = Secp256k1::from((b256::max(), 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff));
    secp256k1.hash(hasher);
    assert_eq(hasher.sha256(), 0x8667e718294e9e0df1d30600ba3eeb201f764aad2dad72748643e4a285e1d1f7);
    assert_eq(sha256(secp256k1), 0x8667e718294e9e0df1d30600ba3eeb201f764aad2dad72748643e4a285e1d1f7);
    assert_eq(hasher.keccak256(), 0xbd8b151773dbbefd7b0df67f2dcc482901728b6df477f4fb2f192733a005d396);
    assert_eq(keccak256(secp256k1), 0xbd8b151773dbbefd7b0df67f2dcc482901728b6df477f4fb2f192733a005d396);
}

#[test()]
fn hash_secp256r1() {
    let mut hasher = Hasher::new();
    let secp256r1 = Secp256r1::from((b256::min(), 0x0000000000000000000000000000000000000000000000000000000000000000));
    secp256r1.hash(hasher);
    assert_eq(hasher.sha256(), 0xf5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b);
    assert_eq(sha256(secp256r1), 0xf5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b);
    assert_eq(hasher.keccak256(), 0xad3228b676f7d3cd4284a5443f17f1962b36e491b30a40b2405849e597ba5fb5);
    assert_eq(keccak256(secp256r1), 0xad3228b676f7d3cd4284a5443f17f1962b36e491b30a40b2405849e597ba5fb5);

    let mut hasher = Hasher::new();
    let secp256r1 = Secp256r1::from((b256::min(), 0x0000000000000000000000000000000000000000000000000000000000000001));
    secp256r1.hash(hasher);
    assert_eq(hasher.sha256(), 0x90f4b39548df55ad6187a1d20d731ecee78c545b94afd16f42ef7592d99cd365);
    assert_eq(sha256(secp256r1), 0x90f4b39548df55ad6187a1d20d731ecee78c545b94afd16f42ef7592d99cd365);
    assert_eq(hasher.keccak256(), 0xa6eef7e35abe7026729641147f7915573c7e97b47efa546f5f6e3230263bcb49);
    assert_eq(keccak256(secp256r1), 0xa6eef7e35abe7026729641147f7915573c7e97b47efa546f5f6e3230263bcb49);

    let mut hasher = Hasher::new();
    let secp256r1 = Secp256r1::from((b256::min(), 0x000000000000000000000000000000000000000000000000000000000000002a));
    secp256r1.hash(hasher);
    assert_eq(hasher.sha256(), 0xc77673a8cc11eb4f660ce1a4ca446423df3b68677ba4c1c1846351ddbeb2e5ef);
    assert_eq(sha256(secp256r1), 0xc77673a8cc11eb4f660ce1a4ca446423df3b68677ba4c1c1846351ddbeb2e5ef);
    assert_eq(hasher.keccak256(), 0x25a1a901705ed15d5376e82511cff743d9474883c82d145cebcc7811e0424a9c);
    assert_eq(keccak256(secp256r1), 0x25a1a901705ed15d5376e82511cff743d9474883c82d145cebcc7811e0424a9c);

    let mut hasher = Hasher::new();
    let secp256r1 = Secp256r1::from((b256::max(), 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff));
    secp256r1.hash(hasher);
    assert_eq(hasher.sha256(), 0x8667e718294e9e0df1d30600ba3eeb201f764aad2dad72748643e4a285e1d1f7);
    assert_eq(sha256(secp256r1), 0x8667e718294e9e0df1d30600ba3eeb201f764aad2dad72748643e4a285e1d1f7);
    assert_eq(hasher.keccak256(), 0xbd8b151773dbbefd7b0df67f2dcc482901728b6df477f4fb2f192733a005d396);
    assert_eq(keccak256(secp256r1), 0xbd8b151773dbbefd7b0df67f2dcc482901728b6df477f4fb2f192733a005d396);
}

#[test()]
fn hash_evm_address() {
    let mut hasher = Hasher::new();
    let evm_address = EvmAddress::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    evm_address.hash(hasher);
    assert_eq(hasher.sha256(), 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);
    assert_eq(sha256(evm_address), 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);
    assert_eq(hasher.keccak256(), 0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563);
    assert_eq(keccak256(evm_address), 0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563);

    let mut hasher = Hasher::new();
    let evm_address = EvmAddress::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    evm_address.hash(hasher);
    assert_eq(hasher.sha256(), 0xec4916dd28fc4c10d78e287ca5d9cc51ee1ae73cbfde08c6b37324cbfaac8bc5);
    assert_eq(sha256(evm_address), 0xec4916dd28fc4c10d78e287ca5d9cc51ee1ae73cbfde08c6b37324cbfaac8bc5);
    assert_eq(hasher.keccak256(), 0xb10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6);
    assert_eq(keccak256(evm_address), 0xb10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6);

    let mut hasher = Hasher::new();
    let evm_address = EvmAddress::from(0x000000000000000000000000000000000000000000000000000000000000002a);
    evm_address.hash(hasher);
    assert_eq(hasher.sha256(), 0x0a28e9ffef0073f9a6a674cf57ee77307f38f0f1bebb087888d9011ed0eeefdf);
    assert_eq(sha256(evm_address), 0x0a28e9ffef0073f9a6a674cf57ee77307f38f0f1bebb087888d9011ed0eeefdf);
    assert_eq(hasher.keccak256(), 0xbeced09521047d05b8960b7e7bcc1d1292cf3e4b2a6b63f48335cbde5f7545d2);
    assert_eq(keccak256(evm_address), 0xbeced09521047d05b8960b7e7bcc1d1292cf3e4b2a6b63f48335cbde5f7545d2);

    let mut hasher = Hasher::new();
    // An EVM address is only 20 bytes, so the first 12 are set to zero.
    let evm_address = EvmAddress::from(0x000000000000000000000000ffffffffffffffffffffffffffffffffffffffff);
    evm_address.hash(hasher);
    assert_eq(hasher.sha256(), 0x78230345cedf8e92525c3cfdb8a95e947de1ed72e881b055dd80f9e523ff33e0);
    assert_eq(sha256(evm_address), 0x78230345cedf8e92525c3cfdb8a95e947de1ed72e881b055dd80f9e523ff33e0);
    assert_eq(hasher.keccak256(), 0xd4e438d33b9d837cd8ac2c60c0ab93462b774f17bb358eb7e74d97f49064fd72);
    assert_eq(keccak256(evm_address), 0xd4e438d33b9d837cd8ac2c60c0ab93462b774f17bb358eb7e74d97f49064fd72);
}

#[test()]
fn hash_b512() {
    let mut hasher = Hasher::new();
    let b512 = B512::from((b256::min(), 0x0000000000000000000000000000000000000000000000000000000000000000));
    b512.hash(hasher);
    assert_eq(hasher.sha256(), 0xf5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b);
    assert_eq(hasher.keccak256(), 0xad3228b676f7d3cd4284a5443f17f1962b36e491b30a40b2405849e597ba5fb5);

    let mut hasher = Hasher::new();
    let b512 = B512::from((b256::min(), 0x0000000000000000000000000000000000000000000000000000000000000001));
    b512.hash(hasher);
    assert_eq(hasher.sha256(), 0x90f4b39548df55ad6187a1d20d731ecee78c545b94afd16f42ef7592d99cd365);
    assert_eq(hasher.keccak256(), 0xa6eef7e35abe7026729641147f7915573c7e97b47efa546f5f6e3230263bcb49);

    let mut hasher = Hasher::new();
    let b512 = B512::from((b256::min(), 0x000000000000000000000000000000000000000000000000000000000000002a));
    b512.hash(hasher);
    assert_eq(hasher.sha256(), 0xc77673a8cc11eb4f660ce1a4ca446423df3b68677ba4c1c1846351ddbeb2e5ef);
    assert_eq(hasher.keccak256(), 0x25a1a901705ed15d5376e82511cff743d9474883c82d145cebcc7811e0424a9c);

    let mut hasher = Hasher::new();
    let b512 = B512::from((b256::max(), 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff));
    b512.hash(hasher);
    assert_eq(hasher.sha256(), 0x8667e718294e9e0df1d30600ba3eeb201f764aad2dad72748643e4a285e1d1f7);
    assert_eq(hasher.keccak256(), 0xbd8b151773dbbefd7b0df67f2dcc482901728b6df477f4fb2f192733a005d396);
}

#[test()]
fn hash_call_params() {
    assert_eq_hashes(
        CallParams {
            coins: 42,
            asset_id: AssetId::from(0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb),
            gas: 112233
        },
        (
            42_u64,
            AssetId::from(0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb),
            112233_u64
        ),
    );
}

#[test()]
fn hash_duration() {
    assert_eq_hashes(
        Duration::seconds(42),
        42_u64,
    );
}

#[test()]
fn hash_time() {
    assert_eq_hashes(
        Time::from(42),
        42_u64,
    );
}

#[test()]
fn hash_u128() {
    assert_eq_hashes(
        U128::from((42, 42)),
        (42_u64, 42_u64),
    );
}

#[cfg(experimental_new_hashing = false)]
#[test()]
fn hash_point2d() {
    assert_eq_hashes(
        Point2D::from([42_u256, 42_u256]),
        (42_u256, 42_u256),
    );
}

#[cfg(experimental_new_hashing = true)]
#[test()]
fn hash_point2d() {
    assert_eq_hashes(
        Point2D::from([42_u256, 42_u256]),
        (32_u64, 42_u256, 32_u64, 42_u256),
    );
}

#[cfg(experimental_new_hashing = false)]
#[test()]
fn hash_scalar() {
    assert_eq_hashes(
        Scalar::from(42_u256),
         42_u256,
    );
}

#[cfg(experimental_new_hashing = true)]
#[test()]
fn hash_scalar() {
    assert_eq_hashes(
        Scalar::from(42_u256),
        (32_u64, 42_u256),
    );
}

#[test()]
fn hash_input() {
    assert_eq_hashes(
        Input::Coin,
        0_u8,
    );
    assert_eq_hashes(
        Input::Contract,
        1_u8,
    );
    assert_eq_hashes(
        Input::Message,
        2_u8,
    );
}

#[test()]
fn hash_output() {
    assert_eq_hashes(
        Output::Coin,
        0_u8,
    );
    assert_eq_hashes(
        Output::Contract,
        1_u8,
    );
    assert_eq_hashes(
        Output::Change,
        2_u8,
    );
    assert_eq_hashes(
        Output::Variable,
        3_u8,
    );
    assert_eq_hashes(
        Output::ContractCreated,
        4_u8,
    );
}

#[test()]
fn hash_transaction() {
    assert_eq_hashes(
        Transaction::Script,
        0_u8,
    );
    assert_eq_hashes(
        Transaction::Create,
        1_u8,
    );
    assert_eq_hashes(
        Transaction::Mint,
        2_u8,
    );
    assert_eq_hashes(
        Transaction::Upgrade,
        3_u8,
    );
    assert_eq_hashes(
        Transaction::Upload,
        4_u8,
    );
    assert_eq_hashes(
        Transaction::Blob,
        5_u8,
    );
}

#[test()]
fn hash_signature() {
    assert_eq_hashes(
        Signature::Secp256k1(Secp256k1::from((b256::min(), 0x000000000000000000000000000000000000000000000000000000000000002a))),
        (
            0_u8,
            (b256::min(), 0x000000000000000000000000000000000000000000000000000000000000002a)
        )
    );
    assert_eq_hashes(
        Signature::Secp256r1(Secp256r1::from((b256::min(), 0x000000000000000000000000000000000000000000000000000000000000002a))),
        (
            1_u8,
            (b256::min(), 0x000000000000000000000000000000000000000000000000000000000000002a)
        )
    );
    assert_eq_hashes(
        Signature::Ed25519(Ed25519::from((b256::min(), 0x000000000000000000000000000000000000000000000000000000000000002a))),
        (
            2_u8,
            (b256::min(), 0x000000000000000000000000000000000000000000000000000000000000002a)
        )
    );
}

#[test()]
fn hash_option() {
    assert_eq_hashes(
        Option::<u8>::None,
        0_u8,
    );
    assert_eq_hashes(
        Option::<u64>::None,
        0_u8,
    );

    assert_eq_hashes(
        Option::<u8>::Some(42),
        (
            1_u8,
            42_u8,
        )
    );
    assert_eq_hashes(
        Option::<u64>::Some(42),
        (
            1_u8,
            42_u64,
        )
    );
}

#[test()]
fn hash_result() {
    assert_eq_hashes(
        Result::<u8, u8>::Ok(42),
        (
            0_u8,
            42_u8,
        )
    );
    assert_eq_hashes(
        Result::<u8, u8>::Err(42),
        (
            1_u8,
            42_u8,
        )
    );

    assert_eq_hashes(
        Result::<u64, u64>::Ok(42),
        (
            0_u8,
            42_u64,
        )
    );
    assert_eq_hashes(
        Result::<u64, u64>::Err(42),
        (
            1_u8,
            42_u64,
        )
    );
}

/// Asserts that two values `a` and `b` hash to the same SHA256 and Keccak256 hashes.
fn assert_eq_hashes<A, B>(a: A, b: B) 
where
    A: Hash,
    B: Hash,
{
    let mut hasher_a = Hasher::new();
    a.hash(hasher_a);
    let sha256_a = hasher_a.sha256();
    let keccak256_a = hasher_a.keccak256();

    let mut hasher_b = Hasher::new();
    b.hash(hasher_b);
    let sha256_b = hasher_b.sha256();
    let keccak256_b = hasher_b.keccak256();

    assert_eq(sha256_a, sha256_b);
    assert_eq(sha256_a, sha256(a));
    assert_eq(sha256_b, sha256(b));

    assert_eq(keccak256_a, keccak256_b);
    assert_eq(keccak256_a, keccak256(a));
    assert_eq(keccak256_b, keccak256(b));
}