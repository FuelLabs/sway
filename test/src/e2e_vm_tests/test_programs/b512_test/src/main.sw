script;

use std::types::B512;
use std::constants::ETH_COLOR;

// helper to prove contiguity of memory in B512 type's hi & lo fields.
fn are_fields_contiguous(big_value: B512) -> bool {
    asm(r1: big_value.hi, r2: big_value.lo, r3, r4, r5, r6) {
        move r3 sp;       // Save a copy of SP in R3.
        cfei i64;         // Reserve 512 bits of stack space.  SP is now R3+64.
        mcpi r3 r1 i64;   // Copy 64 bytes *starting at* big_value.hi.  This should include big_value.lo.
        addi r4 r3 i32;   // Point R4 at where we think big_value.lo was copied to.
        addi r5 zero i32; // Set r5 to 32.  Not sure if there's a better way to do this.  MEQI would be handy.
        meq  r6 r2 r4 r5; // Compare the known big_value.lo in R2 with our copied big_value.lo in r4.
        cfsi i64;         // Free the stack space.
        r6: bool
    }
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
    t1 && t2 && t3 && t4



}