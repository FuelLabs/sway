script;

use std::b512::B512;
use std::address::Address;
use std::ecr::ec_recover;
use std::chain::assert;

fn main() -> bool {
    // the 32 byte address derived from the original private key of '1'.
    // @todo document using the signatures util to generate the address and signature.
    let address: Address = ~Address::from(0x50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0);

    let msg_hash = 0x2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a;

    // full sig: a96368fb96e1f7158d57b45d76bf9cf76927a333cb9ec3b56a50eefe03b2a674
    //           b42c2c99423072cf03e7d0ed21af4e60f2f157985b571c4b5355965579064b6e

    let sig_hi = 0xa96368fb96e1f7158d57b45d76bf9cf76927a333cb9ec3b56a50eefe03b2a674;
    let sig_lo = 0xb42c2c99423072cf03e7d0ed21af4e60f2f157985b571c4b5355965579064b6e;

    // // create a signature
    let signature: B512 = ~B512::from(sig_hi, sig_lo);

    // // recover the address
    let mut recovered_address: Address = ec_recover(signature, msg_hash);
    assert(address.value == 0x50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0);
    assert(recovered_address.value == address.value);

    true
}
