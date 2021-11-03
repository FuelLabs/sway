library ecr;
use ::types::B512;

pub fn ec_recover(signature: B512, msg_hash: b256) -> b256 {

    // let hi = asm(r1, r2: signature.hi, r3: msg_hash) {
    //     ecr r1 r2 r3;
    //     r1
    // };
    // let lo = asm(r1) {
    //     add r1 32;
    //     r1
    // }

    // let pub_key: B512 = B512::new(lo, hi);
    let pub_key = asm(r1, r2: signature.hi, r3: msg_hash) {
        ecr r1 r2 r3;
        r1
    };

    // pointer is a pointer to the start of the 64 byte public key returned by ecr. this could be the hi value of a B512 type, but try to just use pub_key first!
    let address = asm(r1, r2: pub_key , r3: 64) {
        // addi r3 zero i64;
        s256 r1 r2 r3;
        r1: b256
    };
    address


}