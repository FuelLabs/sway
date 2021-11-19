script;
// if test passes, return true

use std::b512::B512;
use std::ecr::ec_recover;

fn main() -> bool {
    // the 32 byte address derived from the original private key. Sha256(pubkey)
    let address: Address = 0x2792dd476211c61c6391d04ff5d6807a85310749000000000000000000000000;

    let message = "Hello from fuel. The comitted number is 42.";

    let sig_hi = 0x4807129506fb24141129677eca31bd65bfc9e6621f937935dc70b00e0e0c31d9;
    let sig_lo = 0x6c54a4c1a8d533bcd465b1882c165a36cdc47f1b2365f4c87be109f9c0430d39;

    // // create a signature
    let signature: B512 = ~B512::from_b256(sig_hi, sig_lo);

    // // hash the message (SHA256(message))
    let msg_hash = 0x623abe7551f140b6b83aefa0cbe5f5254dd0b8115bc83297bba99c871f418886;

    // // recover the address
    let mut recovered_address: Address = ec_recover(signature, msg_hash);

    true
}