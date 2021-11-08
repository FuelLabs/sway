script;

// if test passes, return true

use std::types::B512;
use std::constants::ETH_COLOR;

struct B512 {
    hi: b256,
    lo: b256,
}

impl B512 {
    // Initializes a blank B512
    fn new() -> B512 {
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

    fn from_b_256(hi: b256, lo: b256) -> B512 {
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

        let lo = asm(r1: lo, rlo, r2) {
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

// helper to prove contiguity of memory in B512 type's hi & lo fields.
fn are_fields_aligned(big_value: B512) -> bool {
    let next_bits = asm(r1: big_value.hi, r2, r3, r4: 32) {
        move r1 sp;   // set the stack pointer to start of hi val
        add r3 sp r4; // set r3 to hi + 32 bytes (use addi when implemented)
        move r3 sp;   // move stack pointer to r3
        mcpi r2 r3 i32; // copy the next 32 bytes to r2
        r2: b256      // return what should be lo val
    };
    next_bits == big_value.lo
}

fn main() -> bool {
    let hi_bits: b256 = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
    let lo_bits: b256 = 0x000000000000000000000000000000000000000000000000000000000000002A;
    let zero: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;

    // it allows building from 2 b256's:
    let b: B512 = ~B512::from_b_256(hi_bits, lo_bits);
    let t1 = (b.lo == lo_bits) && (b.hi == hi_bits);

    // it allows creation of new empty type:
    let mut a = ~B512::new();
    let t2 = (a.hi == zero) && (a.lo == zero);

    // // it allows modification of fields:
    a.hi = hi_bits;
    a.lo = lo_bits;
    let t3 =  (a.hi == hi_bits) && (a.lo == lo_bits);

    // it guarantees memory contiguity:
    let mut c = ~B512::new();
    c.hi= hi_bits;
    c.lo = lo_bits;
    let t4 = are_fields_aligned(c);

    // all checks must pass:
    // t1 && t2 && t3 && t4
    t1 && t2 && t4




}