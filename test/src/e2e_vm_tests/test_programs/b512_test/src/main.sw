script;

use std::types::B512;
use std::constants::ETH_COLOR;

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
    let hi_bits: b256 = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFE;
    let lo_bits: b256 = 0x000000000000000000000000000000000000000000000000000000000000002A;
    let modified: b256 = 0x3333333333333333333333333333333333333333333333333333333333333333;
    let zero: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;

    // it allows creation of new empty type:
    let mut a = ~B512::new();
    let t1 = (a.hi == zero) && (a.lo == zero);

    // it allows modification of fields:
    a.hi = hi_bits;
    a.lo = lo_bits;
    let t2 =  (a.hi == hi_bits) && (a.lo == lo_bits);

    // it allows building from 2 b256's:
    let b: B512 = ~B512::from_b_256(hi_bits, lo_bits);
    let t3 = (b.lo == lo_bits) && (b.hi == hi_bits);

    // it allows modification of fields:
    a.hi = modified;
    a.lo = modified;
    let t4 = (a.hi == modified) && (a.lo == modified);


    // it guarantees memory contiguity:
    let mut c = ~B512::new();
    c.hi= hi_bits;
    c.lo = lo_bits;
    let t5 = are_fields_aligned(c);

    // all checks must pass:
    t1 && t2 && t3 && t4 && t5




}