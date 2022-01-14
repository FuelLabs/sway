library ecr;

use ::b512::B512;
use ::address::Address;

/// Recover the public key derived from the private key used to sign a message
pub fn ec_recover(signature: B512, msg_hash: b256) -> B512 {
    let public_key = ~B512::new();

    let hi = asm(buffer, hi: signature.hi, hash: msg_hash) {
        move buffer sp; // Result buffer.
        cfei i64;
        ecr buffer hi hash;
        buffer: b256
    };

    public_key.lo = asm(buffer, hi_ptr: hi, lo_ptr) {
        move buffer sp;
        cfei i32;
        addi lo_ptr hi_ptr i32; // set lo_ptr equal to hi_ptr + 32 bytes
        mcpi buffer lo_ptr i32; // copy 32 bytes starting at lo_ptr into buffer
        buffer: b256
    };

    public_key.hi = hi;
    public_key
}

/// Recover the address derived from the private key used to sign a message
pub fn ec_recover_address(signature: B512, msg_hash: b256) -> Address {
    let address = asm(pub_key_buffer, sig_ptr: signature.hi, hash: msg_hash, addr_buffer, sixty_four: 64) {
        move pub_key_buffer sp; // mv sp to pub_key result buffer.
        cfei i64;
        ecr pub_key_buffer sig_ptr hash; // recover public_key from sig & hash
        move addr_buffer sp; // mv sp to addr result buffer.
        cfei i32;
        s256 addr_buffer pub_key_buffer sixty_four; // hash 64 bytes to the addr_buffer
        addr_buffer: b256
    };

    ~Address::from(address)
}
