library ecr;
use ::types::B512;

/// Recover the address of the private key used to sign a message
// @todo change return type to `Address`
pub fn ec_recover(signature: B512, msg_hash: b256) -> b256 {

    // store the first 32 bytes of the pubkey in hi
    let hi = asm(r1, hi: signature.hi, hash: msg_hash) {
        ecr r1 hi hash;
        r1: u64
    };

    // store the last 32 bytes of the pubkey in lo
    let lo = asm(r1, r2: hi) {
        add r1 32;
        r1
    }

    // @todo switch to use `from()` when implemented
    let pub_key: B512 = B512::from_b256(lo, hi);


    // pointer is a pointer to the start of the 64 byte public key returned by ecr. this could be the hi value of a B512 type, but try to just use pub_key first!
    let address = asm(r1, r2: pub_key , r3: 64) {
        // addi r3 zero i64;
        s256 r1 r2 r3;
        r1: b256
    };
    address


}

// let pub_key = asm(r1, r2: signature.hi, r3: msg_hash) {
    //     ecr r1 r2 r3;
    //     r1
    // };