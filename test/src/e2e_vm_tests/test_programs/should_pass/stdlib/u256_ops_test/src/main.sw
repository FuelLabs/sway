script;

use std::u256::U256;

fn main() -> bool {
    let first = U256::from((0, 0, 0, 0));
    let second = U256::from((0, 0, 0, 1));
    let max_u64 = U256::from((0, 0, 0, u64::max()));

    let one = first + second;

    assert(one.c == 0);
    assert(one.d == 1);

    let two = one + one;
    assert(two.c == 0);
    assert(two.d == 2);

    let add_of_one = max_u64 + one;
    assert(add_of_one.c == 1);
    assert(add_of_one.d == 0);

    let add_of_two = max_u64 + two;
    assert(add_of_two.c == 1);
    assert(add_of_two.d == 1);

    let add_max = max_u64 + max_u64;
    assert(add_max.c == 1);
    assert(add_max.d == u64::max() - 1);

    let sub_one = second - first;
    assert(sub_one.c == 0);
    assert(sub_one.d == 1);

    let sub_zero = first - first;
    assert(sub_zero.c == 0);
    assert(sub_zero.d == 0);

    let sub_max_again = add_of_two - two;
    assert(sub_max_again.c == 0);
    assert(sub_max_again.d == u64::max());

    let zero_in_between = U256::from((0, 1, 0, 10));
    let d_nonzero = U256::from((0, 0, 0, 12));

    let res = zero_in_between - d_nonzero;

    assert(res.c == u64::max());
    assert(res.d == u64::max() - 1);

    let one_upper = U256::from((0, 0, 1, 0));

    let right_shift_one_upper = one_upper >> 1;
    assert(right_shift_one_upper.c == 0);
    assert(right_shift_one_upper.d == (1 << 63));

    let left_shift_one_upper_right_shift = right_shift_one_upper << 1;
    assert(left_shift_one_upper_right_shift == one_upper);

    let one_left_shift_64 = one << 64;
    assert(one_left_shift_64.c == 1);
    assert(one_left_shift_64.d == 0);

    let three_left_shift_one = U256::from((0, 0, 0, 3)) << 1;
    assert(three_left_shift_one.c == 0);
    assert(three_left_shift_one.d == 6);

    let c_max_left_shift_one = U256::from((0, u64::max(), 0, 0)) >> 1;

    assert(c_max_left_shift_one.b == (1 << 63) - 1);
    assert(c_max_left_shift_one.c == 1 << 63);
    assert(c_max_left_shift_one.d == 0);

    let last_left_shift_one = U256::from((1, 0, 0, 0)) >> 1;
    assert(last_left_shift_one.a == 0);
    assert(last_left_shift_one.b == 1 << 63);

    let not_1_0_0_0 = !U256::from((1, 0, 0, 0));
    assert(not_1_0_0_0.a == u64::max() - 1);
    assert(not_1_0_0_0.b == u64::max());
    assert(not_1_0_0_0.c == u64::max());
    assert(not_1_0_0_0.d == u64::max());

    let not_0_1_0_0 = !U256::from((0, 1, 0, 0));
    assert(not_0_1_0_0.a == u64::max());
    assert(not_0_1_0_0.b == u64::max() - 1);
    assert(not_0_1_0_0.c == u64::max());
    assert(not_0_1_0_0.d == u64::max());

    let not_0_0_1_0 = !U256::from((0, 0, 1, 0));
    assert(not_0_0_1_0.a == u64::max());
    assert(not_0_0_1_0.b == u64::max());
    assert(not_0_0_1_0.c == u64::max() - 1);
    assert(not_0_0_1_0.d == u64::max());

    let not_0_0_0_1 = !U256::from((0, 0, 0, 1));
    assert(not_0_0_0_1.a == u64::max());
    assert(not_0_0_0_1.b == u64::max());
    assert(not_0_0_0_1.c == u64::max());
    assert(not_0_0_0_1.d == u64::max() - 1);

    let not_1_1_0_0 = !U256::from((1, 1, 0, 0));
    assert(not_1_1_0_0.a == u64::max() - 1);
    assert(not_1_1_0_0.b == u64::max() - 1);
    assert(not_1_1_0_0.c == u64::max());
    assert(not_1_1_0_0.d == u64::max());

    let not_0_1_1_0 = !U256::from((0, 1, 1, 0));
    assert(not_0_1_1_0.a == u64::max());
    assert(not_0_1_1_0.b == u64::max() - 1);
    assert(not_0_1_1_0.c == u64::max() - 1);
    assert(not_0_1_1_0.d == u64::max());

    let not_0_0_1_1 = !U256::from((0, 0, 1, 1));
    assert(not_0_0_1_1.a == u64::max());
    assert(not_0_0_1_1.b == u64::max());
    assert(not_0_0_1_1.c == u64::max() - 1);
    assert(not_0_0_1_1.d == u64::max() - 1);

    let not_1_0_1_0 = !U256::from((1, 0, 1, 0));
    assert(not_1_0_1_0.a == u64::max() - 1);
    assert(not_1_0_1_0.b == u64::max());
    assert(not_1_0_1_0.c == u64::max() - 1);
    assert(not_1_0_1_0.d == u64::max());

    let not_1_0_0_1 = !U256::from((1, 0, 0, 1));
    assert(not_1_0_0_1.a == u64::max() - 1);
    assert(not_1_0_0_1.b == u64::max());
    assert(not_1_0_0_1.c == u64::max());
    assert(not_1_0_0_1.d == u64::max() - 1);

    let not_0_1_0_1 = !U256::from((0, 1, 0, 1));
    assert(not_0_1_0_1.a == u64::max());
    assert(not_0_1_0_1.b == u64::max() - 1);
    assert(not_0_1_0_1.c == u64::max());
    assert(not_0_1_0_1.d == u64::max() - 1);

    let not_1_1_1_0 = !U256::from((1, 1, 1, 0));
    assert(not_1_1_1_0.a == u64::max() - 1);
    assert(not_1_1_1_0.b == u64::max() - 1);
    assert(not_1_1_1_0.c == u64::max() - 1);
    assert(not_1_1_1_0.d == u64::max());

    let not_0_1_1_1 = !U256::from((0, 1, 1, 1));
    assert(not_0_1_1_1.a == u64::max());
    assert(not_0_1_1_1.b == u64::max() - 1);
    assert(not_0_1_1_1.c == u64::max() - 1);
    assert(not_0_1_1_1.d == u64::max() - 1);

    let not_1_0_1_1 = !U256::from((1, 0, 1, 1));
    assert(not_1_0_1_1.a == u64::max() - 1);
    assert(not_1_0_1_1.b == u64::max());
    assert(not_1_0_1_1.c == u64::max() - 1);
    assert(not_1_0_1_1.d == u64::max() - 1);

    let not_1_1_0_1 = !U256::from((1, 1, 0, 1));
    assert(not_1_1_0_1.a == u64::max() - 1);
    assert(not_1_1_0_1.b == u64::max() - 1);
    assert(not_1_1_0_1.c == u64::max());
    assert(not_1_1_0_1.d == u64::max() - 1);

    let not_1_1_1_1 = !U256::from((1, 1, 1, 1));
    assert(not_1_1_1_1.a == u64::max() - 1);
    assert(not_1_1_1_1.b == u64::max() - 1);
    assert(not_1_1_1_1.c == u64::max() - 1);
    assert(not_1_1_1_1.d == u64::max() - 1);

    true
}
