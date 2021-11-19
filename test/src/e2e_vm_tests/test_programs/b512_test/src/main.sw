script;

use std::b512::B512;
use std::constants::ETH_COLOR;
use std::chain::assert;

// helper to prove contiguity of memory in B512 type's hi & lo fields.
fn are_fields_contiguous(big_value: B512) -> bool {
    asm(r1: big_value.hi, r2: big_value.lo, r3, r4, r5, r6) {
        move r3 sp;       // Save a copy of SP in R3.
        cfei i64;         // Reserve 512 bits of stack space.  SP is now R3+64.
        mcpi r3 r1 i64;   // Copy 64 bytes *starting at* big_value.hi (includes big_value.lo)
        addi r4 r3 i32;   // Point R4 at where we think big_value.lo was copied to.
        addi r5 zero i32; // Set r5 to 32.
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
    assert((a.hi == zero) && (a.lo == zero));

    // it allows reassignment of fields:
    a.hi = hi_bits;
    a.lo = lo_bits;
    assert((a.hi == hi_bits) && (a.lo == lo_bits));

    // it allows building from 2 b256's:
    let mut b = ~B512::from(hi_bits, lo_bits);
    assert((b.hi == hi_bits) && (b.lo == lo_bits));

    // it allows reassignment of fields:
    b.hi = modified;
    b.lo = modified;
    assert((b.hi == modified) && (b.lo == modified));

    // it guarantees memory contiguity:
    let mut c = ~B512::new();
    c.hi= hi_bits;
    c.lo = lo_bits;
    assert(are_fields_contiguous(c));

    true
}