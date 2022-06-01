library b256_ops;

use ::flags::{disable_panic_on_overflow, enable_panic_on_overflow};
use ::core::ops::Add;
use utils::compose::*;
use ::logging::log;

impl Add for b256 {
    fn add(self, other: Self) -> Self {
        let (s1, s2, s3, s4) = decompose(self);
        let (o1, o2, o3, o4) = decompose(other);
        let mut total_overflow = 0;

        let (sum_4, ovf_4) = overflowing_add(s4, o4);
        let (sum_3, ovf_3) = overflowing_add(s3, o3);
        total_overflow = ovf_3 + ovf_4;
        let (sum_3_final, carry_3) = overflowing_add(sum_3, total_overflow);
        let (sum_2, ovf_2) = overflowing_add(s2, o2);
        total_overflow = ovf_2 + carry_3;
        let (sum_2_final, carry_2) = overflowing_add(sum_2, total_overflow);
        let (sum_1, ovf_1) = overflowing_add(s1, o1);
        total_overflow = ovf_1 + carry_2;

        compose(sum_1 + total_overflow, sum_2_final, sum_3_final, sum_4)
    }

}

/// This is used to get both the sum and the overflow value from an addition.
/// With normal addition, any overflow will cause a vm panic.
fn overflowing_add(a: u64, b: u64) -> (u64, u64) {
    disable_panic_on_overflow ();
    let mut result = (0u64, 0u64);
    asm(sum, overflow, a: a, b: b, result_ptr: result) {
        // Add left and right.
        add sum a b;
        // Immediately copy the overflow of the addition from `$of` into
        // `overflow` so that it's not lost.
        move overflow of;
        // Store the sum into the first word of result.
        sw result_ptr sum i0;
        // Store the overflow into the second word of result.
        sw result_ptr overflow i1;
    };
    enable_panic_on_overflow();
    result
}
