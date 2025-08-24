script;

use std::u128::*;

fn main() -> bool {
    let first = U128::from((0, 0));
    let second = U128::from((0, 1));
    let max_u64 = U128::from((0, u64::max()));

    let one = first + second;
    assert(one.upper() == 0);
    assert(one.lower() == 1);

    let two = one + one;
    assert(two.upper() == 0);
    assert(two.lower() == 2);

    let add_of_one = max_u64 + one;
    assert(add_of_one.upper() == 1);
    assert(add_of_one.lower() == 0);

    let add_of_two = max_u64 + two;
    assert(add_of_two.upper() == 1);
    assert(add_of_two.lower() == 1);

    let add_max = max_u64 + max_u64;
    assert(add_max.upper() == 1);
    assert(add_max.lower() == u64::max() - 1);

    let sub_one = second - first;
    assert(sub_one.upper() == 0);
    assert(sub_one.lower() == 1);

    let sub_zero = first - first;
    assert(sub_zero.upper() == 0);
    assert(sub_zero.lower() == 0);

    let sub_max_again = add_of_two - two;
    assert(sub_max_again.upper() == 0);
    assert(sub_max_again.lower() == u64::max());

    let mul_four = 2.overflowing_mul(2);
    assert(mul_four.upper() == 0);
    assert(mul_four.lower() == 4);

    let mul_eight = 4.overflowing_mul(2);
    assert(mul_eight.upper() == 0);
    assert(mul_eight.lower() == 8);

    let mul_of_two = u64::max().overflowing_mul(2);
    assert(mul_of_two.upper() == 1);
    assert(mul_of_two.lower() == u64::max() - 1);

    let mul_of_four = u64::max().overflowing_mul(4);
    assert(mul_of_four.upper() == 3);
    assert(mul_of_four.lower() == u64::max() - 3);

    let mul_max = u64::max().overflowing_mul(u64::max());
    assert(mul_max.upper() == u64::max() - 1);
    assert(mul_max.lower() == 1);

    let one_upper = U128::from((1, 0));

    let right_shift_one_upper = one_upper >> 1;
    assert(right_shift_one_upper.upper() == 0);
    assert(right_shift_one_upper.lower() == (1 << 63));

    let left_shift_one_upper_right_shift = right_shift_one_upper << 1;
    assert(left_shift_one_upper_right_shift == one_upper);

    let one_left_shift_64 = one << 64;
    assert(one_left_shift_64.upper() == 1);
    assert(one_left_shift_64.lower() == 0);

    let three_left_shift_one = U128::from((0, 3)) << 1;
    assert(three_left_shift_one.upper() == 0);
    assert(three_left_shift_one.lower() == 6);

    let not_0_3 = !U128::from((0, 3));
    assert(not_0_3.upper() == u64::max());
    assert(not_0_3.lower() == u64::max() - 3);

    let not_3_3 = !U128::from((3, 3));
    assert(not_3_3.upper() == u64::max() - 3);
    assert(not_3_3.lower() == u64::max() - 3);

    let not_3_0 = !U128::from((3, 0));
    assert(not_3_0.upper() == u64::max() - 3);
    assert(not_3_0.lower() == u64::max());

    // test try_as_u64()
    let eleven = U128::from((0, 11));
    let unwrapped = eleven.try_as_u64().unwrap();
    assert(unwrapped == 11);

    let err_1 = U128::from((42, 11)).try_as_u64();
    assert(match err_1 {
        Err(U128Error::LossOfPrecision) => {
            true
        },
        _ => {
            false
        },
    });

    let err_1 = U128::from((42, 0)).try_as_u64();
    assert(match err_1 {
        Err(U128Error::LossOfPrecision) => {
            true
        },
        _ => {
            false
        },
    });

    true
}
