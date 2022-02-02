library ecr;

use ::b512::B512;
use ::address::Address;

/// Recover the public key derived from the private key used to sign a message
pub fn ec_recover(signature: B512, msg_hash: b256) -> B512 {
    let public_key = ~B512::new();

    asm(buffer: public_key.bytes, sig: signature.bytes, hash: msg_hash) {
        ecr buffer sig hash;
    };

    public_key
}

/// Recover the address derived from the private key used to sign a message
pub fn ec_recover_address(signature: B512, msg_hash: b256) -> Address {
    let address = asm(sig: signature.bytes, hash: msg_hash, addr_buffer, pub_key_buffer, hash_len: 64) {
        move addr_buffer sp; // Buffer for address.
        cfei i32;
        move pub_key_buffer sp; // Temporary buffer for recovered key.
        cfei i64;
        ecr pub_key_buffer sig hash; // Recover public_key from sig & hash.
        s256 addr_buffer pub_key_buffer hash_len; // Hash 64 bytes to the addr_buffer.
        cfsi i64; // Free temporary key buffer.
        addr_buffer: b256
    };

    ~Address::from(address)
}
