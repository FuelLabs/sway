script;

use std::b512::*;

// helper to prove contiguity of memory in B512 type's hi & lo fields.
fn are_fields_contiguous(big_value: B512) -> bool {
    asm(r1: (big_value.bits())[0], r2: (big_value.bits())[1], r3, r4, r5, r6) {
        move r3 sp; // Save a copy of SP in R3.
        cfei i64; // Reserve 512 bits of stack space.  SP is now R3+64.
        mcpi r3 r1 i64; // Copy 64 bytes *starting at* big_value.hi (includes big_value.lo)
        addi r4 r3 i32; // Point R4 at where we think big_value.lo was copied to.
        addi r5 zero i32; // Set r5 to 32.
        meq r6 r2 r4 r5; // Compare the known big_value.lo in R2 with our copied big_value.lo in r4.
        cfsi i64; // Free the stack space.
        r6: bool
    }
}

fn main() -> bool {
    let zero = b256::min();
    let hi_bits: b256 = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFE;
    let lo_bits: b256 = 0x000000000000000000000000000000000000000000000000000000000000002A;
    let modified: b256 = 0x3333333333333333333333333333333333333333333333333333333333333333;

    // it allows creation of new empty type:
    let mut a = B512::new();
    assert(((a.bits())[0] == zero) && ((a.bits())[1] == zero));

    // it allows building from 2 b256's:
    let mut b = B512::from((hi_bits, lo_bits));
    assert(((b.bits())[0] == hi_bits) && ((b.bits())[1] == lo_bits));

    // it guarantees memory contiguity:
    let mut c = B512::from((hi_bits, lo_bits));
    assert(are_fields_contiguous(c));

    // it allows direct comparison of equality:
    let one = B512::from((hi_bits, modified));
    let two = B512::from((hi_bits, modified));
    let three = B512::from((modified, hi_bits));
    let four = B512::from((lo_bits, modified));
    assert(one == two);
    assert(one != three);
    assert(one != four);

    let one_tuple:(b256,b256) = one.into();
    let two_tuple:(b256,b256) = two.into();
    let three_tuple:(b256,b256) = three.into();
    let four_tuple:(b256,b256) = four.into();

    assert(one_tuple.0 == hi_bits);
    assert(one_tuple.1 == modified);
    assert(two_tuple.0 == hi_bits);
    assert(two_tuple.1 == modified);
    assert(three_tuple.0 == modified);
    assert(three_tuple.1 == hi_bits);
    assert(four_tuple.0 == lo_bits);
    assert(four_tuple.1 == modified);

    true
}
