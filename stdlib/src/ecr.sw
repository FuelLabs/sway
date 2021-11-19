library ecr;

// @todo fix import ! method won't work, Could not find symbol "from_b256" in this scope.
use ::b512::*;

// @note temp copy from b512 lib until imports fixed.
pub struct B512 {
    hi: b256,
    lo: b256,
}

impl B512 {
    // Initializes a blank B512
  pub fn new() -> B512 {
        let hi = asm(rhi) {
            move rhi sp;
            cfei i32;
            rhi: b256
        };

        let lo = asm(rlo) {
            move rlo sp;
            cfei i32;
            rlo: b256
        };

        B512 {
            hi: hi,
            lo: lo
        }
    }

   pub fn from_b256(hi: b256, lo: b256) -> B512 {
        // copy the two given b256s into contiguous stack memory
        // this involves grabbing the stack pointer, extending the stack by 256 bits,
        // using MCP to copy hi into first ptr
        // repeat w/ second ptr

        let hi = asm(r1: hi, rhi) {
            move rhi sp; // move stack pointer to rhi
            cfei i32;  // extend call frame by 32 bytes to allocate more memory. now $rhi is pointing to blank, uninitialized (but allocated) memory
            mcpi rhi r1 i32; // refactor to use mcpi when implemented!
            rhi: b256
        };

        let lo = asm(r1: lo, rlo) {
            move rlo sp;
            cfei i32;
            // now $rlo is pointing to blank memory that we can use
            mcpi rlo r1 i32; // refactor to use mcpi when implemented!
            rlo: b256
        };

        B512 {
            hi: hi,
            lo: lo
        }
    }
}



/// Recover the address of the private key used to sign a message
// @todo change return type to `Address`
pub fn ec_recover(signature: B512, msg_hash: b256) -> b256 {

    // store the first 32 bytes of the pubkey in hi
    let hi = asm(r1, hi: signature.hi, hash: msg_hash) {
        ecr r1 hi hash;
        r1: b256
    };

    // store the last 32 bytes of the pubkey in lo
    let lo = asm(r1: hi, r2) {
        addi r2 r1 i32; // add 32 bytes to hi location
        move r2 sp; // move stack pointer to hi + 32
        r2: b256  // return the next 32 bytes
    };

    // @todo switch to use `from()` when implemented
    let pub_key: B512 = ~B512::from_b256(hi, lo);

    let address = asm(r1, r2: pub_key.hi , r3: 64) {
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


// fn pub_key_pointer(sig: B512, hash: b256) -> b256 {
    //     asm(r1, hi: sig.hi, hash: hash) {
    //         ecr r1 hi hash;
    //         r1: b256
    //     }
    // }

// instead of initializing r2 with pub_key, can I pass a function as a value?
    // let address = asm(address, r2: pub_key_pointer(signature, msg_hash) , r3: 64) {
    //     s256 address r2 r3;
    //     address: b256
    // };