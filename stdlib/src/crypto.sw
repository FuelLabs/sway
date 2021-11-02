library crypto;

struct B512 {
    lo: b256,
    hi: b256
}

impl B512 {
    fn new(lo: b256, hi: b256) -> B512 {

        let mut new_memory_slot = asm(r1: 64) {
            aloc r1;
            r1
        }


        B512 { lo, hi}
    }
}

// MEM[$rA, 64] = ecrecover(MEM[$rB, 64], MEM[$rC, 32]);

// This means that from $rA to $rA + 64 contains the return value. You can access that by loading $rA into hi and $rA + 32 into lo, essentially chunking it back in half.

pub fn ec_recover(signature: b512, digest: b256) -> b256 {
    let hi = asm(r1, r2: signature, r3: digest) {
        ecr r1 r2 r3;
        r1
    };
    let lo = asm(r1) {
        add r1 32;
        r1
    }

    let pub_key: B512 = B512::new(lo, hi);

    hash_pair(pub_key::lo, pub_key::hi, HashMethod::Sha256)

}