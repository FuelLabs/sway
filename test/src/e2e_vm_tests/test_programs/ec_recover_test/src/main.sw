script;

use std::b512::B512;
use std::address::Address;
use std::ecr::ec_recover;
use std::ecr::recover_pubkey;
use std::chain::assert;

fn main() -> bool {

   //======================================================
   // test data from sig-gen-util:

//    private key: 0x0000000000000000000000000000000000000000000000000000000000000001
// public key:79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8

// pubkey: [4, 121, 190, 102, 126, 249, 220, 187, 172, 85, 160, 98, 149, 206, 135, 11, 7, 2, 155, 252, 219, 45, 206, 40, 217, 89, 242, 129, 91, 22, 248, 23, 152, 72, 58, 218, 119, 38, 163, 196, 101, 93, 164, 251, 252, 14, 17, 8, 168, 253, 23, 180, 72, 166, 133, 84, 25, 156, 71, 208, 143, 251, 16, 212, 184]

// hashed_msg: 0x9c100ea4273e9cf8c65a3dfddc0f3d37a2ed4ad29b6ae914445dee722f82efd5
// msg: 0x2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a
// Address: "0xaf533e027d9d6ccc0958c470c26ea9d96a9b76fd8cb5737f8d38353c03b5d7a4"

// Full Signature: RecoverableSignature(a96368fb96e1f7158d57b45d76bf9cf76927a333cb9ec3b56a50eefe03b2a674b42c2c99423072cf03e7d0ed21af4e60f2f157985b571c4b5355965579064b6e01)

// Serialized Signature: (RecoveryId(1), [116, 166, 178, 3, 254, 238, 80, 106, 181, 195, 158, 203, 51, 163, 39, 105, 247, 156, 191, 118, 93, 180, 87, 141, 21, 247, 225, 150, 251, 104, 99, 169, 110, 75, 6, 121, 85, 150, 85, 83, 75, 28, 87, 91, 152, 87, 241, 242, 96, 78, 175, 33, 237, 208, 231, 3, 207, 114, 48, 66, 153, 44, 44, 180])




    let pubkey: B512 = ~B512::from(0x79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798, 0x483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8);

    let address: Address = ~Address::from(0xaf533e027d9d6ccc0958c470c26ea9d96a9b76fd8cb5737f8d38353c03b5d7a4);

    let msg_hash = 0x2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a;

    let sig_hi = 0xa96368fb96e1f7158d57b45d76bf9cf76927a333cb9ec3b56a50eefe03b2a674;
    let sig_lo = 0xb42c2c99423072cf03e7d0ed21af4e60f2f157985b571c4b5355965579064b6e;

    // create a signature:
    let signature: B512 = ~B512::from(sig_hi, sig_lo);

    // recover the address:
    let mut recovered_address: Address = ec_recover(signature, msg_hash);
    // let mut recovered_pubkey: B512 = recover_pubkey(signature, msg_hash);

    // assert(recovered_pubkey.hi == pubkey.hi);
    // assert(recovered_pubkey.lo == pubkey.lo);
    assert(recovered_address.value == address.value);

    true
}
