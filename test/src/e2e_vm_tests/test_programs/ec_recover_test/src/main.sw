script;

use std::b512::B512;
use std::address::Address;
use std::ecr::ec_recover;
use std::chain::assert;

fn main() -> bool {
    // the 32 byte address derived from the original private key of '1'.
    // @todo
    let address: Address = ~Address::from(0x50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0);

    let msg_hash = 0x2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a;

    // full sig: 74a6b203feee506ab5c39ecb33a32769f79cbf765db4578d15f7e196fb6863a96e4b0679559655534b1c575b9857f1f2604eaf21edd0e703cf723042992c2cb4
    let sig_hi = 0x74a6b203feee506ab5c39ecb33a32769f79cbf765db4578d15f7e196fb6863a9;
    let sig_lo = 0x6e4b0679559655534b1c575b9857f1f2604eaf21edd0e703cf723042992c2cb4;

    // // create a signature
    let signature: B512 = ~B512::from(sig_hi, sig_lo);

    // // recover the address
    let mut recovered_address: Address = ec_recover(signature, msg_hash);
    assert(recovered_address.value == address.value);

    true
}