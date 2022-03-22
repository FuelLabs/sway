script;

use std::b512::B512;
use std::address::Address;
use std::ecr::ec_recover_address;
use std::ecr::ec_recover;
use std::chain::assert;

fn main() -> bool {

   //======================================================
   // test data from sig-gen-util: /sway/sig_gen_util/src/main.rs
   /**
   Secret Key: SecretKey(3b940b5586823dfd02ae3b461bb4336b5ecbaefd6627aa922efc048fec0c881c)
   Public Key: 1d152307c6b72b0ed0418b0e70cd80e7f5295b8d86f5722d3f5213fbd2394f36
               b7ce9c3e45905178455900b44abb308f3ef480481a4b2ee3f70aca157fde396a
   Address: 0x6ba48099f6b75cae5a403863ace6ee8dc03f75e7aebc58b819667477358ae677
   Message Hash: 0x8ddb13a2ab58f413bd3121e1ddc8b83a328f3b830d19a7c471f0be652d23bb0e
   Signature: 82115ed208d8fe8dd522d88ca77812b34d270d6bb6326ff511297766a3af1166
              c07204f554a00e49a2ee69f0979dc4feef07f7dba8d779d388fb2a53bc9bcde4
   */

    let pubkey: B512 = ~B512::from(0x1d152307c6b72b0ed0418b0e70cd80e7f5295b8d86f5722d3f5213fbd2394f36, 0xb7ce9c3e45905178455900b44abb308f3ef480481a4b2ee3f70aca157fde396a);

    let address: Address = ~Address::from(0x6ba48099f6b75cae5a403863ace6ee8dc03f75e7aebc58b819667477358ae677);

    let msg_hash = 0x8ddb13a2ab58f413bd3121e1ddc8b83a328f3b830d19a7c471f0be652d23bb0e;

    let sig_hi = 0x82115ed208d8fe8dd522d88ca77812b34d270d6bb6326ff511297766a3af1166;
    let sig_lo = 0xc07204f554a00e49a2ee69f0979dc4feef07f7dba8d779d388fb2a53bc9bcde4;

    // create a signature:
    let signature: B512 = ~B512::from(sig_hi, sig_lo);

    // recover the address:
    let recovered_address: Address = ec_recover_address(signature, msg_hash);
    assert(recovered_address.value == address.value);

    // recover the public key:
    let recovered_pubkey: B512 = ec_recover(signature, msg_hash);
    assert((recovered_pubkey.bytes)[0] == (pubkey.bytes)[0]);
    assert((recovered_pubkey.bytes)[1] == (pubkey.bytes)[1]);

    true
}
