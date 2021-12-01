library ecr;

use ::b512::B512;
use ::address::Address;

/// Recover the address derived from the private key used to sign a message
pub fn ec_recover(signature: B512, msg_hash: b256) -> Address {

    // we know that the B512 type's inner values are contiguous in memory.
    // the ERC OpCode descriptions states:
    // "The 64-byte public key (x, y) recovered from 64-byte signature starting at $rB on 32-byte message hash starting at $rC"
    // Store the first 32 bytes of the public key in hi_bits:
    let hi_bits = asm(r1, hi: signature.hi, hash: msg_hash) {
        ecr r1 hi hash;
        r1: b256
    };

    // Store the last 32 bytes of the public key in lo:
    let lo_bits = asm(r1: hi_bits, r2) {
        addi r2 r1 i32; // add 32 bytes to hi location
        move r2 sp; // move stack pointer to hi + 32
        cfei i32;
        r2: b256  // return the next 32 bytes
    };

    let pub_key: B512 = ~B512::from(hi_bits, lo_bits);

    let address = asm(r1, r2: pub_key.hi , r3: 64) {
        s256 r1 r2 r3;
        r1: b256
    };

    ~Address::from(address)
}
