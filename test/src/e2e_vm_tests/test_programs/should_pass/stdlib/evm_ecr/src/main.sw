script;

use std::assert::assert;
use std::b512::B512;
use std::ecr::EcRecoverError;
use std::result::*;
use std::vm::evm::evm_address::EvmAddress;
use std::vm::evm::ecr::ec_recover_evm_address;

fn main() -> bool {
    //======================================================
    // test data from sig-gen-util: /sway/sig_gen_util/src/main.rs
    /**
    Secret Key: SecretKey(3b940b5586823dfd02ae3b461bb4336b5ecbaefd6627aa922efc048fec0c881c)
    Public Key: 1d152307c6b72b0ed0418b0e70cd80e7f5295b8d86f5722d3f5213fbd2394f36b7ce9c3e45905178455900b44abb308f3ef480481a4b2ee3f70aca157fde396a
    Fuel Address (sha2-256): 6ba48099f6b75cae5a403863ace6ee8dc03f75e7aebc58b819667477358ae677
    EVM pubkey hash (keccak256): e4eab8f844a8d11b205fd137a1b7ea5ede26f651909505d99cf8b5c0d4c8e9c1
    Message Hash: 8ddb13a2ab58f413bd3121e1ddc8b83a328f3b830d19a7c471f0be652d23bb0e
    Signature: 82115ed208d8fe8dd522d88ca77812b34d270d6bb6326ff511297766a3af1166c07204f554a00e49a2ee69f0979dc4feef07f7dba8d779d388fb2a53bc9bcde4
   */

    // Get the expected EVM address
    let pubkeyhash = 0xe4eab8f844a8d11b205fd137a1b7ea5ede26f651909505d99cf8b5c0d4c8e9c1;
    let evm_address = EvmAddress::from(pubkeyhash);

    let msg_hash = 0x8ddb13a2ab58f413bd3121e1ddc8b83a328f3b830d19a7c471f0be652d23bb0e;

    // create a signature:
    let sig_hi = 0x82115ed208d8fe8dd522d88ca77812b34d270d6bb6326ff511297766a3af1166;
    let sig_lo = 0xc07204f554a00e49a2ee69f0979dc4feef07f7dba8d779d388fb2a53bc9bcde4;
    let signature: B512 = B512::from(sig_hi, sig_lo);

    // recover the address:
    let result: Result<EvmAddress, EcRecoverError> = ec_recover_evm_address(signature, msg_hash);
    let recovered_address = result.unwrap();

    recovered_address == evm_address
}
