library;

use std::{
    flags::{
        disable_panic_on_overflow,
        disable_panic_on_unsafe_math,
        set_flags,
    },
    registers::flags,
    u128::U128,
};

#[test]
fn u128_from_u8() {
    let u8_1: u8 = u8::min();
    let u8_2: u8 = u8::max();
    let u8_3: u8 = 1u8;

    let u128_1 = <U128 as From<u8>>::from(u8_1);
    let u128_2 = <U128 as From<u8>>::from(u8_2);
    let u128_3 = <U128 as From<u8>>::from(u8_3);

    assert(u128_1.as_u64().unwrap() == 0u64);
    assert(u128_2.as_u64().unwrap() == 255u64);
    assert(u128_3.as_u64().unwrap() == 1u64);
}

#[test]
fn u128_u8_into() {
    let u8_1: u8 = u8::min();
    let u8_2: u8 = u8::max();
    let u8_3: u8 = 1u8;

    let u128_1: U128 = u8_1.into();
    let u128_2: U128 = u8_2.into();
    let u128_3: U128 = u8_3.into();

    assert(u128_1.as_u64().unwrap() == 0u64);
    assert(u128_2.as_u64().unwrap() == 255u64);
    assert(u128_3.as_u64().unwrap() == 1u64);
}

#[test]
fn u128_from_u16() {
    let u16_1: u16 = u16::min();
    let u16_2: u16 = u16::max();
    let u16_3: u16 = 1u16;

    let u128_1 = <U128 as From<u16>>::from(u16_1);
    let u128_2 = <U128 as From<u16>>::from(u16_2);
    let u128_3 = <U128 as From<u16>>::from(u16_3);

    assert(u128_1.as_u64().unwrap() == 0u64);
    assert(u128_2.as_u64().unwrap() == 65535u64);
    assert(u128_3.as_u64().unwrap() == 1u64);
}

#[test]
fn u128_u16_into() {
    let u16_1: u16 = u16::min();
    let u16_2: u16 = u16::max();
    let u16_3: u16 = 1u16;

    let u128_1: U128 = u16_1.into();
    let u128_2: U128 = u16_2.into();
    let u128_3: U128 = u16_3.into();

    assert(u128_1.as_u64().unwrap() == 0u64);
    assert(u128_2.as_u64().unwrap() == 65535u64);
    assert(u128_3.as_u64().unwrap() == 1u64);
}

#[test]
fn u128_from_u32() {
    let u32_1: u32 = u32::min();
    let u32_2: u32 = u32::max();
    let u32_3: u32 = 1u32;

    let u128_1 = <U128 as From<u32>>::from(u32_1);
    let u128_2 = <U128 as From<u32>>::from(u32_2);
    let u128_3 = <U128 as From<u32>>::from(u32_3);

    assert(u128_1.as_u64().unwrap() == 0u64);
    assert(u128_2.as_u64().unwrap() == 4294967295u64);
    assert(u128_3.as_u64().unwrap() == 1u64);
}

#[test]
fn u128_u32_into() {
    let u32_1: u32 = u32::min();
    let u32_2: u32 = u32::max();
    let u32_3: u32 = 1u32;

    let u128_1: U128 = u32_1.into();
    let u128_2: U128 = u32_2.into();
    let u128_3: U128 = u32_3.into();

    assert(u128_1.as_u64().unwrap() == 0u64);
    assert(u128_2.as_u64().unwrap() == 4294967295u64);
    assert(u128_3.as_u64().unwrap() == 1u64);
}

#[test]
fn u128_from_u64() {
    let u64_1: u64 = u64::min();
    let u64_2: u64 = u64::max();
    let u64_3: u64 = 1u64;

    let u128_1 = <U128 as From<u64>>::from(u64_1);
    let u128_2 = <U128 as From<u64>>::from(u64_2);
    let u128_3 = <U128 as From<u64>>::from(u64_3);

    assert(u128_1.as_u64().unwrap() == 0u64);
    assert(u128_2.as_u64().unwrap() == 18446744073709551615u64);
    assert(u128_3.as_u64().unwrap() == 1u64);
}

#[test]
fn u128_u64_into() {
    let u64_1: u64 = u64::min();
    let u64_2: u64 = u64::max();
    let u64_3: u64 = 1u64;

    let u128_1: U128 = u64_1.into();
    let u128_2: U128 = u64_2.into();
    let u128_3: U128 = u64_3.into();

    assert(u128_1.as_u64().unwrap() == 0u64);
    assert(u128_2.as_u64().unwrap() == 18446744073709551615u64);
    assert(u128_3.as_u64().unwrap() == 1u64);
}

#[test]
fn u128_from_u64_tuple() {
    let u64_1: u64 = u64::min();
    let u64_2: u64 = u64::max();
    let u64_3: u64 = 1u64;

    let u128_1 = <U128 as From<(u64, u64)>>::from((u64_1, u64_1));
    let u128_2 = <U128 as From<(u64, u64)>>::from((u64_2, u64_2));
    let u128_3 = <U128 as From<(u64, u64)>>::from((u64_3, u64_3));

    assert(u128_1.upper() == 0u64);
    assert(u128_1.lower() == 0u64);
    assert(u128_2.upper() == 18446744073709551615u64);
    assert(u128_2.lower() == 18446744073709551615u64);
    assert(u128_3.upper() == 1u64);
    assert(u128_3.lower() == 1u64);
}

#[test]
fn u128_into_u64_tuple() {
    // Glob operator needed for U128.into()
    use std::u128::*;

    let u128_1 = U128::from((0u64, 0u64));
    let u128_2 = U128::from((18446744073709551615u64, 18446744073709551615u64));
    let u128_3 = U128::from((1u64, 1u64));

    let u64_1: (u64, u64) = u128_1.into();
    let u64_2: (u64, u64) = u128_2.into();
    let u64_3: (u64, u64) = u128_3.into();

    assert(u64_1.0 == u64::min());
    assert(u64_1.1 == u64::min());
    assert(u64_2.0 == u64::max());
    assert(u64_2.1 == u64::max());
    assert(u64_3.0 == 1u64);
    assert(u64_3.1 == 1u64);
}

#[test]
fn u128_u64_tuple_into() {
    // Glob operator needed for From<U128> for (u64, u64)
    use std::u128::*;

    let u64_1: (u64, u64) = (u64::min(), u64::min());
    let u64_2: (u64, u64) = (u64::max(), u64::max());
    let u64_3: (u64, u64) = (1u64, 1u64);

    let u128_1: U128 = u64_1.into();
    let u128_2: U128 = u64_2.into();
    let u128_3: U128 = u64_3.into();

    assert(u128_1.upper() == 0u64);
    assert(u128_1.lower() == 0u64);
    assert(u128_2.upper() == 18446744073709551615u64);
    assert(u128_2.lower() == 18446744073709551615u64);
    assert(u128_3.upper() == 1u64);
    assert(u128_3.lower() == 1u64);
}

#[test]
fn u128_u64_tuple_from() {
    // Glob operator needed for From<U128> for (u64, u64)
    use std::u128::*;

    let u128_1 = U128::from((0u64, 0u64));
    let u128_2 = U128::from((18446744073709551615u64, 18446744073709551615u64));
    let u128_3 = U128::from((1u64, 1u64));

    let u64_1: (u64, u64) = <(u64, u64) as From<U128>>::from(u128_1);
    let u64_2: (u64, u64) = <(u64, u64) as From<U128>>::from(u128_2);
    let u64_3: (u64, u64) = <(u64, u64) as From<U128>>::from(u128_3);

    assert(u64_1.0 == u64::min());
    assert(u64_1.1 == u64::min());
    assert(u64_2.0 == u64::max());
    assert(u64_2.1 == u64::max());
    assert(u64_3.0 == 1u64);
    assert(u64_3.1 == 1u64);
}

#[test]
fn u128_eq() {
    let u64_1: u64 = u64::min();
    let u64_2: u64 = u64::max();
    let u64_3: u64 = 1u64;

    let u128_1 = <U128 as From<(u64, u64)>>::from((u64_1, u64_1));
    let u128_2 = <U128 as From<(u64, u64)>>::from((u64_1, u64_1));
    let u128_3 = <U128 as From<(u64, u64)>>::from((u64_2, u64_2));
    let u128_4 = <U128 as From<(u64, u64)>>::from((u64_2, u64_2));
    let u128_5 = <U128 as From<(u64, u64)>>::from((u64_3, u64_3));
    let u128_6 = <U128 as From<(u64, u64)>>::from((u64_3, u64_3));

    assert(u128_1 == u128_2);
    assert(u128_3 == u128_4);
    assert(u128_5 == u128_6);
}

#[test]
fn u128_ne() {
    let u64_1: u64 = u64::min();
    let u64_2: u64 = u64::max();
    let u64_3: u64 = 1u64;

    let u128_1 = <U128 as From<(u64, u64)>>::from((u64_1, u64_1));
    let u128_2 = <U128 as From<(u64, u64)>>::from((u64_1, u64_2));
    let u128_3 = <U128 as From<(u64, u64)>>::from((u64_2, u64_2));
    let u128_4 = <U128 as From<(u64, u64)>>::from((u64_2, u64_3));
    let u128_5 = <U128 as From<(u64, u64)>>::from((u64_3, u64_3));
    let u128_6 = <U128 as From<(u64, u64)>>::from((u64_3, u64_1));

    assert(u128_1 != u128_2);
    assert(u128_1 != u128_3);
    assert(u128_1 != u128_4);
    assert(u128_1 != u128_5);
    assert(u128_1 != u128_6);

    assert(u128_2 != u128_3);
    assert(u128_2 != u128_4);
    assert(u128_2 != u128_5);
    assert(u128_2 != u128_6);

    assert(u128_3 != u128_4);
    assert(u128_3 != u128_5);
    assert(u128_3 != u128_6);

    assert(u128_4 != u128_5);
    assert(u128_4 != u128_6);

    assert(u128_5 != u128_6);
}

#[test]
fn u128_ord() {
    let u64_1: u64 = u64::min();
    let u64_2: u64 = u64::max();
    let u64_3: u64 = 1u64;

    let u128_1 = <U128 as From<(u64, u64)>>::from((u64_1, u64_1)); // 0, 0
    let u128_2 = <U128 as From<(u64, u64)>>::from((u64_1, u64_3)); // 0, 1
    let u128_3 = <U128 as From<(u64, u64)>>::from((u64_3, u64_1)); // 1, 0
    let u128_4 = <U128 as From<(u64, u64)>>::from((u64_2, u64_2)); // max, max
    let u128_5 = <U128 as From<(u64, u64)>>::from((u64_3, u64_2)); // 1, max
    let u128_6 = <U128 as From<(u64, u64)>>::from((u64_2, u64_1)); // max, 0
    let u128_7 = <U128 as From<(u64, u64)>>::from((u64_3, u64_3)); // 1, 1
    let u128_8 = <U128 as From<(u64, u64)>>::from((u64_2, u64_3)); // max, 1
    let u128_9 = <U128 as From<(u64, u64)>>::from((u64_1, u64_2)); // 0, max
    assert(u128_1 < u128_2);
    assert(u128_3 > u128_1);
    assert(u128_3 > u128_2);

    assert(u128_4 > u128_1);
    assert(u128_4 > u128_2);
    assert(u128_4 > u128_3);
    assert(u128_4 > u128_5);
    assert(u128_4 > u128_6);
    assert(u128_4 > u128_7);
    assert(u128_4 > u128_8);
    assert(u128_4 > u128_9);

    assert(u128_5 > u128_1);
    assert(u128_5 > u128_2);
    assert(u128_5 > u128_3);
    assert(u128_5 < u128_6);
    assert(u128_5 > u128_7);
    assert(u128_5 < u128_8);
    assert(u128_5 > u128_9);

    assert(u128_6 > u128_1);
    assert(u128_6 > u128_2);
    assert(u128_6 > u128_3);
    assert(u128_6 > u128_7);
    assert(u128_6 < u128_8);
    assert(u128_6 > u128_9);

    assert(u128_7 > u128_1);
    assert(u128_7 > u128_2);
    assert(u128_7 > u128_3);
}

#[test]
fn u128_new() {
    let new_u128 = U128::new();
    assert(new_u128.upper() == 0);
    assert(new_u128.lower() == 0);
}

#[test]
fn u128_as_u64() {
    let u64_1: u64 = u64::min();
    let u64_2: u64 = u64::max();
    let u64_3: u64 = 1u64;

    let u128_1 = <U128 as From<u64>>::from(u64_1);
    let u128_2 = <U128 as From<u64>>::from(u64_2);
    let u128_3 = <U128 as From<u64>>::from(u64_3);

    assert(u128_1.as_u64().unwrap() == 0u64);
    assert(u128_2.as_u64().unwrap() == 18446744073709551615u64);
    assert(u128_3.as_u64().unwrap() == 1u64);

    let u128_4 = <U128 as From<(u64, u64)>>::from((u64_3, u64_1));
    let u128_5 = <U128 as From<(u64, u64)>>::from((u64_2, u64_1));
    let u128_6 = <U128 as From<(u64, u64)>>::from((u64_2, u64_2));

    assert(u128_4.as_u64().is_err());
    assert(u128_5.as_u64().is_err());
    assert(u128_6.as_u64().is_err());
}

#[test]
fn u128_min() {
    let new_u128 = U128::min();
    assert(new_u128.upper() == 0);
    assert(new_u128.lower() == 0);
}

#[test]
fn u128_max() {
    let new_u128 = U128::max();
    assert(new_u128.upper() == u64::max());
    assert(new_u128.lower() == u64::max());
}

#[test]
fn u128_bits() {
    assert(U128::bits() == 128u32);
}

#[test]
fn u128_upper() {
    let u64_1: u64 = u64::min();
    let u64_2: u64 = u64::max();
    let u64_3: u64 = 1u64;

    let u128_1 = <U128 as From<(u64, u64)>>::from((u64_1, u64_1));
    let u128_2 = <U128 as From<(u64, u64)>>::from((u64_2, u64_2));
    let u128_3 = <U128 as From<(u64, u64)>>::from((u64_3, u64_3));

    assert(u128_1.upper() == 0u64);
    assert(u128_2.upper() == 18446744073709551615u64);
    assert(u128_3.upper() == 1u64);
}

#[test]
fn u128_lower() {
    let u64_1: u64 = u64::min();
    let u64_2: u64 = u64::max();
    let u64_3: u64 = 1u64;

    let u128_1 = <U128 as From<(u64, u64)>>::from((u64_1, u64_1));
    let u128_2 = <U128 as From<(u64, u64)>>::from((u64_2, u64_2));
    let u128_3 = <U128 as From<(u64, u64)>>::from((u64_3, u64_3));

    assert(u128_1.lower() == 0u64);
    assert(u128_2.lower() == 18446744073709551615u64);
    assert(u128_3.lower() == 1u64);
}

#[test]
fn u128_zero() {
    let zero_u128 = U128::zero();
    assert(zero_u128.as_u64().unwrap() == 0u64);
}

#[test]
fn u128_is_zero() {
    let zero_u128 = U128::zero();
    assert(zero_u128.is_zero());

    let other1_u128 = U128::from((0, 1));
    assert(!other1_u128.is_zero());

    let other2_u128 = U128::from((1, 0));
    assert(!other2_u128.is_zero());

    let other3_u128 = U128::from((1, 1));
    assert(!other3_u128.is_zero());

    let other4_u128 = U128::from((0, 0));
    assert(other4_u128.is_zero());
}

#[test]
fn u128_bitwise_and() {
    let one = U128::from((0, 1));
    let two = U128::from((0, 2));
    let three = U128::from((0, 3));
    let thirteen = U128::from((0, 13));
    let one_upper = U128::from((1, 0));
    let zero = U128::zero();

    assert(one & two == zero);
    assert(one & three == one);
    assert(one & thirteen == one);
    assert(one & one_upper == zero);

    assert(two & three == two);
    assert(two & thirteen == zero);
    assert(two & one_upper == zero);

    assert(three & thirteen == one);
    assert(three & one_upper == zero);

    assert(thirteen & one_upper == zero);

    assert(one & one == one);
    assert(two & two == two);
    assert(three & three == three);
    assert(thirteen & thirteen == thirteen);
    assert(one_upper & one_upper == one_upper);
}

#[test]
fn u128_bitwise_or() {
    let one = U128::from((0, 1));
    let two = U128::from((0, 2));
    let three = U128::from((0, 3));
    let thirteen = U128::from((0, 13));
    let one_upper = U128::from((1, 0));

    assert(one | two == three);
    assert(one | three == three);
    assert(one | thirteen == thirteen);
    assert(one | one_upper == one_upper + one);

    assert(two | three == three);
    assert(two | thirteen == thirteen + two);
    assert(two | one_upper == one_upper + two);

    assert(three | thirteen == thirteen + two);
    assert(three | one_upper == one_upper + three);

    assert(thirteen | one_upper == one_upper + thirteen);

    assert(one | one == one);
    assert(two | two == two);
    assert(three | three == three);
    assert(thirteen | thirteen == thirteen);
    assert(one_upper | one_upper == one_upper);
}

#[test]
fn u128_shift() {
    let one = U128::from((0, 1));
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
}

#[test]
fn u128_not() {
    let not_0_3 = !U128::from((0, 3));
    assert(not_0_3.upper() == u64::max());
    assert(not_0_3.lower() == u64::max() - 3);

    let not_3_3 = !U128::from((3, 3));
    assert(not_3_3.upper() == u64::max() - 3);
    assert(not_3_3.lower() == u64::max() - 3);

    let not_3_0 = !U128::from((3, 0));
    assert(not_3_0.upper() == u64::max() - 3);
    assert(not_3_0.lower() == u64::max());
}

#[test]
fn u128_add() {
    let first = U128::from((0, 0));
    let second = U128::from((0, 1));
    let max_u128 = U128::from((0, u64::max()));

    let one = first + second;
    assert(one.upper() == 0);
    assert(one.lower() == 1);

    let two = one + one;
    assert(two.upper() == 0);
    assert(two.lower() == 2);

    let add_of_one = max_u128 + one;
    assert(add_of_one.upper() == 1);
    assert(add_of_one.lower() == 0);

    let add_of_two = max_u128 + two;
    assert(add_of_two.upper() == 1);
    assert(add_of_two.lower() == 1);

    let add_max = max_u128 + max_u128;
    assert(add_max.upper() == 1);
    assert(add_max.lower() == u64::max() - 1);
}

#[test(should_revert)]
fn revert_u128_add() {
    let one = U128::from((0, 1));
    let max_u128 = U128::from((u64::max(), u64::max()));

    let _result = one + max_u128;
}

#[test(should_revert)]
fn revert_u128_add_on_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let one = U128::from((0, 1));
    let max_u128 = U128::from((u64::max(), u64::max()));

    let _result = one + max_u128;
}

#[test]
fn u128_add_overflow() {
    let _ = disable_panic_on_overflow();
    let one = U128::from((0, 1));
    let two = U128::from((0, 2));
    let max_u128 = U128::from((u64::max(), u64::max()));

    let res_1 = one + max_u128;
    assert(res_1 == U128::zero());

    let res_2 = max_u128 + two;
    assert(res_2 == one);

    let a = U128::max();
    let b = U128::from((0, 1));
    let c = a + b;

    assert(c == U128::from((0, 0)));
}

#[test]
fn u128_sub() {
    let first = U128::from((0, 0));
    let second = U128::from((0, 1));
    let two = U128::from((0, 2));
    let max_u64 = U128::from((0, u64::max()));

    let sub_one = second - first;
    assert(sub_one.upper() == 0);
    assert(sub_one.lower() == 1);

    let sub_zero = first - first;
    assert(sub_zero.upper() == 0);
    assert(sub_zero.lower() == 0);

    let add_of_two = max_u64 + two;
    let sub_max_again = add_of_two - two;
    assert(sub_max_again.upper() == 0);
    assert(sub_max_again.lower() == u64::max());
}

#[test(should_revert)]
fn revert_u128_sub() {
    let first = U128::from((0, 0));
    let second = U128::from((0, 1));

    let _result = first - second;
}

#[test(should_revert)]
fn revert_u128_sub_on_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let first = U128::from((0, 0));
    let second = U128::from((0, 1));

    let _result = first - second;
}

#[test]
fn u128_sub_underflow() {
    let _ = disable_panic_on_overflow();
    let first = U128::from((0, 0));
    let second = U128::from((0, 1));

    let result = first - second;
    assert(result == U128::max());

    let a = U128::from((0, 1));
    let b = U128::from((0, 2));
    let c = a - b;

    assert(c == U128::max());
}

#[test]
fn u128_multiply() {
    let two = U128::from((0, 2));
    let max_u64 = U128::from((0, u64::max()));

    let mul_128_of_two = max_u64 * two;
    assert(mul_128_of_two.upper() == 1);
    assert(mul_128_of_two.lower() == u64::max() - 1);

    let mul_128_of_four = mul_128_of_two * two;
    assert(mul_128_of_four.upper() == 3);
    assert(mul_128_of_four.lower() == u64::max() - 3);

    let mul_128_max = max_u64 * max_u64;
    assert(mul_128_max.upper() == u64::max() - 1);
    assert(mul_128_max.lower() == 1);

    let upper_u128 = U128::from((1, 0));
    assert(upper_u128 * U128::from(2u64) == U128::from((2,0)));
}

#[test(should_revert)]
fn revert_u128_multiply() {
    let first = U128::from((0, 2));
    let second = U128::from((u64::max(), 1));

    let result = first * second;
    log(result);
}

#[test(should_revert)]
fn revert_u128_multiply_on_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let first = U128::from((0, 2));
    let second = U128::from((u64::max(), 1));

    let _result = first * second;
}

#[test]
fn u128_multiply_overflow() {
    let _ = disable_panic_on_overflow();
    let first = U128::from((0, 3));
    let second = U128::max();

    let result = first * second;
    assert(result == U128::from((18446744073709551615, 18446744073709551613)));

    let a = U128::max();
    let b = U128::from((0, 2));
    let c = a * b;

    // 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFE
    assert(c == U128::from((0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFE)));
}

#[test]
fn u128_divide() {
    let two = U128::from((0, 2));
    let max_u64 = U128::from((0, u64::max()));

    let div_max_two = max_u64 / two;
    assert(div_max_two.upper() == 0);
    assert(div_max_two.lower() == u64::max() >> 1);

    // Product of u64::MAX and u64::MAX.
    let dividend = U128::from((u64::max() - 1, 1));
    let div_max_max = dividend / max_u64;
    assert(div_max_max.upper() == 0);
    assert(div_max_max.lower() == u64::max());
}

#[test(should_revert)]
fn revert_u128_divide_by_zero() {
    let first = U128::from((0, 1));
    let second = U128::from((0, 0));

    let _result = first / second;
}

#[test(should_revert)]
fn revert_u128_divide_by_zero_disabled_overflow() {
    let _ = disable_panic_on_overflow();
    let first = U128::from((0, 1));
    let second = U128::from((0, 0));

    let _result = first / second;
}

#[test]
fn u128_divide_by_zero_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let first = U128::from((0, 1));
    let second = U128::from((0, 0));

    let _result = first / second;
}

#[test]
fn u128_pow() {
    let mut u_128 = U128::from((0, 7));
    let mut pow_of_u_128 = u_128.pow(2u32);
    assert(pow_of_u_128 == U128::from((0, 49)));

    pow_of_u_128 = u_128.pow(3u32);
    assert(pow_of_u_128 == U128::from((0, 343)));

    u_128 = U128::from((0, 3));
    pow_of_u_128 = u_128.pow(2u32);
    assert(pow_of_u_128 == U128::from((0, 9)));

    u_128 = U128::from((0, 5));
    pow_of_u_128 = u_128.pow(2u32);
    assert(pow_of_u_128 == U128::from((0, 25)));

    pow_of_u_128 = u_128.pow(7u32);
    assert(pow_of_u_128 == U128::from((0, 78125)));

    u_128 = U128::from((0, 8));
    pow_of_u_128 = u_128.pow(2u32);
    assert(pow_of_u_128 == U128::from((0, 64)));

    pow_of_u_128 = u_128.pow(9u32);
    assert(pow_of_u_128 == U128::from((0, 134217728)));

    u_128 = U128::from((0, 10));
    pow_of_u_128 = u_128.pow(2u32);
    assert(pow_of_u_128 == U128::from((0, 100)));

    pow_of_u_128 = u_128.pow(5u32);
    assert(pow_of_u_128 == U128::from((0, 100000)));

    u_128 = U128::from((0, 12));
    pow_of_u_128 = u_128.pow(2u32);
    assert(pow_of_u_128 == U128::from((0, 144)));

    pow_of_u_128 = u_128.pow(3u32);
    assert(pow_of_u_128 == U128::from((0, 1728)));

    // Test reassignment
    u_128 = U128::from((0, 13));
    u_128 = u_128.pow(1u32);
    assert(u_128 == U128::from((0, 13)));

    let max_u64_u128 = U128::from(u64::max());
    let max_pow = max_u64_u128.pow(2);
    let expected_result = U128::from((18446744073709551614, 1));
    assert(max_pow == expected_result);

    let u128_upper_and_lower_not_zero = U128::from((1, 1));
    let upper_and_lower_result = u128_upper_and_lower_not_zero.pow(1);
    assert(upper_and_lower_result == u128_upper_and_lower_not_zero);
}

#[test(should_revert)]
fn revert_u128_pow_overflow() {
    let max_u64_u128 = U128::from(u64::max());
    let max_pow = max_u64_u128.pow(3);
    log(max_pow);
}

#[test]
fn u128_root() {
    let mut u_128: U128 = U128::from((0, 49));
    let mut root_of_u_128 = u_128.sqrt();

    assert(root_of_u_128 == U128::from((0, 7)));

    u_128 = U128::from((0, 25));
    root_of_u_128 = u_128.sqrt();
    assert(root_of_u_128 == U128::from((0, 5)));

    u_128 = U128::from((0, 81));
    root_of_u_128 = u_128.sqrt();
    assert(root_of_u_128 == U128::from((0, 9)));

    u_128 = U128::from((0, 144));
    root_of_u_128 = u_128.sqrt();
    assert(root_of_u_128 == U128::from((0, 12)));

    u_128 = U128::from((0, 1));
    root_of_u_128 = u_128.sqrt();
    assert(root_of_u_128 == U128::from((0, 1)));
}

#[test(should_revert)]
fn revert_u128_zero_root() {
    let zero = U128::zero();

    let _result = zero.sqrt();
}

#[test(should_revert)]
fn revert_u128_zero_root_overflow_disabled() {
    let _ = disable_panic_on_overflow();
    let zero = U128::zero();

    let _result = zero.sqrt();
}

#[test]
fn u128_zero_root_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let zero = U128::zero();

    let result = zero.sqrt();
    assert(result == zero);
}

#[test]
fn u128_mod() {
    let u128_zero = U128::zero();
    let u128_1 = U128::from((0, 1));
    let u128_2 = U128::from((0, 2));
    let u128_3 = U128::from((0, 3));
    let u128_max = U128::max();

    assert(u128_zero % u128_1 == u128_zero);
    assert(u128_zero % u128_2 == u128_zero);
    assert(u128_zero % u128_3 == u128_zero);
    assert(u128_zero % u128_max == u128_zero);

    assert(u128_1 % u128_1 == u128_zero);
    assert(U128::from((0, 10)) % u128_1 == u128_zero);
    assert(U128::from((0, 10000)) % u128_1 == u128_zero);
    assert(u128_max % u128_1 == u128_zero);

    assert(u128_2 % u128_2 == u128_zero);
    assert(u128_3 % u128_2 == u128_1);
    assert(U128::from((0, 10)) % u128_2 == u128_zero);
    assert(U128::from((0, 10000)) % u128_2 == u128_zero);
    assert(U128::from((0, 10001)) % u128_2 == u128_1);
    assert(U128::from((0, u64::max())) % u128_2 == u128_1);
    assert(U128::from((u64::max(), 0)) % u128_2 == u128_zero);
    assert(U128::from((u64::max(), 1)) % u128_2 == u128_1);
    assert(u128_max % u128_2 == u128_1);

    assert(u128_3 % u128_3 == u128_zero);
    assert(u128_2 % u128_3 == u128_2);
    assert(u128_1 % u128_3 == u128_1);
    assert(U128::from((0, 30000)) % u128_3 == u128_zero);
    assert(U128::from((0, 30001)) % u128_3 == u128_1);
    assert(U128::from((0, 30002)) % u128_3 == u128_2);
    assert(U128::from((u64::max(), 0)) % u128_3 == u128_zero);
    assert(U128::from((u64::max(), 1)) % u128_3 == u128_1);
    assert(U128::from((u64::max(), 2)) % u128_3 == u128_2);
    assert(u128_max % u128_3 == u128_zero);

    assert(U128::from((u64::max(), 0)) % U128::from((u64::max(), 0)) == u128_zero);
    assert(U128::from((u64::max(), 1)) % U128::from((u64::max(), 0)) == u128_1);
    assert(U128::from((u64::max(), 2)) % U128::from((u64::max(), 0)) == u128_2);
    assert(U128::from((u64::max(), 3)) % U128::from((u64::max(), 0)) == u128_3);
    assert(u128_max % U128::from((u64::max(), 0)) == U128::from((0, u64::max())));
}

#[test(should_revert)]
fn revert_u128_mod_zero() {
    let a = U128::from((0, 1));
    let b = U128::zero();

    let result = a % b;
}

#[test(should_revert)]
fn revert_u128_mod_zero_disabled_overflow() {
    let _ = disable_panic_on_overflow();
    let a = U128::from((0, 1));
    let b = U128::zero();

    let result = a % b;
}

#[test]
fn u128_mod_zero_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let a = U128::from((0, 1));
    let b = U128::zero();

    let result = a % b;
    assert(result == a);
}

#[test]
fn u128_log() {
    let u_128_0: U128 = U128::from((0, 0));
    let u_128_1: U128 = U128::from((0, 1));
    let u_128_2: U128 = U128::from((0, 2));
    let u_128_3: U128 = U128::from((0, 3));
    let u_128_6: U128 = U128::from((0, 6));
    let u_128_8: U128 = U128::from((0, 8));
    let u_128_9: U128 = U128::from((0, 9));
    let u_128_10: U128 = U128::from((0, 10));
    let u_128_20: U128 = U128::from((0, 20));
    let u_128_42: U128 = U128::from((0, 42));
    let u_128_64: U128 = U128::from((0, 64));
    let u_128_100: U128 = U128::from((0, 100));
    let u_128_127: U128 = U128::from((0, 127));
    let u64_max_times_two: U128 = U128::from((1, 0));
    let u_128_max: U128 = U128::max();

    assert(u_128_2.log(u_128_2) == u_128_1);
    assert(u_128_1.log(u_128_3) == u_128_0);
    assert(u_128_8.log(u_128_2) == u_128_3);
    assert(u_128_100.log(u_128_10) == u_128_2);
    assert(u_128_100.log(u_128_2) == u_128_6);
    assert(u_128_100.log(u_128_9) == u_128_2);
    assert(u_128_max.log(u_128_2) == u_128_127);
    assert(u_128_max.log(u_128_9) == u_128_42);
    assert(u64_max_times_two.log(u_128_2) == u_128_64);
    assert(u64_max_times_two.log(u_128_9) == u_128_20);
}

#[test]
fn u128_log_unsafe_math() {
    let prior_flags = disable_panic_on_unsafe_math();

    let before_flags = flags();

    let zero = U128::from(0_u64);
    let one = U128::from(1_u64);
    let three = U128::from(3_u64);

    assert(one.log(one) == zero);
    assert(zero.log(three) == zero);

    assert(before_flags == flags());

    set_flags(prior_flags);
}

#[test(should_revert)]
fn revert_u128_1log1() {
    let res = U128::from(1_u64).log(U128::from(1_u64));
    log(res);
}

#[test(should_revert)]
fn revert_u128_disable_overflow() {
    let res = U128::from(1_u64).log(U128::from(1_u64));
    log(res);
}

#[test(should_revert)]
fn revert_unsafe_math_u128_0log_3() {
    let res = U128::from(0_u64).log(U128::from(3_u64));
    log(res);
}

#[test(should_revert)]
fn revert_u128_1log1_disable_overflow() {
    let _ = disable_panic_on_overflow();
    let res = U128::from(1_u64).log(U128::from(1_u64));
    log(res);
}

#[test(should_revert)]
fn revert_u128_disable_overflow_disable_overflow() {
    let _ = disable_panic_on_overflow();
    let res = U128::from(1_u64).log(U128::from(1_u64));
    log(res);
}

#[test(should_revert)]
fn revert_unsafe_math_u128_0log_3_disable_overflow() {
    let _ = disable_panic_on_overflow();
    let res = U128::from(0_u64).log(U128::from(3_u64));
    log(res);
}

#[test]
fn u128_binary_log() {
    let u_128_1: U128 = U128::from((0, 1));
    let u_128_2: U128 = U128::from((0, 2));
    let u_128_3: U128 = U128::from((0, 3));
    let u_128_6: U128 = U128::from((0, 6));
    let u_128_8: U128 = U128::from((0, 8));
    let u_128_64: U128 = U128::from((0, 64));
    let u_128_127: U128 = U128::from((0, 127));
    let u_128_100: U128 = U128::from((0, 100));
    let u_128_max_div_2: U128 = U128::from((1, 0));
    let u_128_max: U128 = U128::max();

    assert(u_128_2.log2() == u_128_1);
    assert(u_128_8.log2() == u_128_3);
    assert(u_128_100.log2() == u_128_6);
    assert(u_128_max.log2() == u_128_127);
    assert(u_128_max_div_2.log2() == u_128_64);
}

#[test(should_revert)]
fn revert_u128_binary_log() {
    let u_128_0: U128 = U128::from((0, 0));

    let _result = u_128_0.log2();
}

#[test(should_revert)]
fn revert_u128_binary_log_disable_overflow() {
    let _ = disable_panic_on_overflow();
    let u_128_0: U128 = U128::from((0, 0));

    let _result = u_128_0.log2();
}

#[test]
fn u128_unsafe_math_log2() {
    let prior_flags = disable_panic_on_unsafe_math();
    // 0 is not a valid operand for log2
    let a = U128::zero();
    let res = a.log2();

    assert(res == U128::zero());

    set_flags(prior_flags);
}

#[test]
fn parity_u128_log_with_ruint() {
    let prior_flags = flags();

    // Failure cases found by comparing parity with ruint implementation of U128
    // https://docs.rs/ruint/latest/src/ruint/log.rs.html#45-89
    let a = [
        2, 4, 4, 4, 4, 5, 5, 5, 6, 6, 7, 8, 8, 8, 8, 8, 8, 8, 8, 9, 9, 9, 9, 9, 9,
        9, 10, 10, 10, 10, 10, 10, 11, 11, 11, 11, 11, 12, 12, 12, 12, 13, 13, 13,
        14, 14, 15, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16,
        16, 16, 16, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17,
        17, 17, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18,
        19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 20, 20, 20,
        20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 21, 21, 21, 21, 21, 21, 21,
        21, 21, 21, 21, 21, 21, 21, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22,
        22, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 24, 24, 24, 24, 24, 24,
        24, 24, 24, 24, 24, 25, 25, 25, 25, 25, 25, 25, 25, 25, 26, 26, 26, 26, 26,
        26, 26, 26, 27, 27, 27, 27, 27, 27, 27, 28, 28, 28, 28, 28, 28, 29, 29, 29,
        29, 29, 30, 30, 30, 30, 31, 31, 31, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
        32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
        32, 32, 32, 32, 32, 33, 33, 33, 33, 33, 33, 33, 33, 33, 33, 33, 33, 33, 33,
        33, 33, 33, 33, 33, 33, 33, 33, 33, 33, 33, 33, 33, 33, 33, 33, 33, 33, 33,
        34, 34, 34, 34, 34, 34, 34, 34, 34, 34, 34, 34, 34, 34, 34, 34, 34, 34, 34,
        34, 34, 34, 34, 34, 34, 34, 34, 34, 34, 34, 34, 34, 35, 35, 35, 35, 35, 35,
        35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35,
        35, 35, 35, 35, 35, 35, 36, 36, 36, 36, 36, 36, 36, 36, 36, 36, 36, 36, 36,
        36, 36, 36, 36, 36, 36, 36, 36, 36, 36, 36, 36, 36, 36, 36, 36, 37, 37, 37,
        37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37, 37,
        37, 37, 37, 37, 37, 37, 38, 38, 38, 38, 38, 38, 38, 38, 38, 38, 38, 38, 38,
        38, 38, 38, 38, 38, 38, 38, 38, 38, 38, 38, 38, 38, 38, 39, 39, 39, 39, 39,
        39, 39, 39, 39, 39, 39, 39, 39, 39, 39, 39, 39, 39, 39, 39, 39, 39, 39, 39,
        39, 39, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40, 40,
        40, 40, 40, 40, 40, 40, 40, 40, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41,
        41, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41, 42, 42, 42, 42, 42, 42,
        42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 43, 43,
        43, 43, 43, 43, 43, 43, 43, 43, 43, 43, 43, 43, 43, 43, 43, 43, 43, 43, 43,
        43, 44, 44, 44, 44, 44, 44, 44, 44, 44, 44, 44, 44, 44, 44, 44, 44, 44, 44,
        44, 44, 44, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45,
        45, 45, 45, 45, 46, 46, 46, 46, 46, 46, 46, 46, 46, 46, 46, 46, 46, 46, 46,
        46, 46, 46, 46, 47, 47, 47, 47, 47, 47, 47, 47, 47, 47, 47, 47, 47, 47, 47,
        47, 47, 47, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
        48, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 50, 50, 50,
        50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 51, 51, 51, 51, 51, 51, 51, 51,
        51, 51, 51, 51, 51, 52, 52, 52, 52, 52, 52, 52, 52, 52, 52, 52, 52, 53, 53,
        53, 53, 53, 53, 53, 53, 53, 53, 53, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54,
        55, 55, 55, 55, 55, 55, 55, 55, 55, 56, 56, 56, 56, 56, 56, 56, 56, 57, 57,
        57, 57, 57, 57, 57, 58, 58, 58, 58, 58, 58, 59, 59, 59, 59, 59, 60, 60, 60,
        60, 61, 61, 61, 62, 62, 63, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
        64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
        64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 65, 65, 65,
        65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
        65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
        65, 65, 65, 65, 65, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66,
        66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66,
        66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 67, 67, 67, 67, 67, 67, 67,
        67, 67, 67, 67, 67, 67, 67, 67, 67, 67, 67, 67, 67, 67, 67, 67, 67, 67, 67,
        67, 67, 67, 67, 67, 67, 67, 67, 67, 67, 67, 67, 67, 67, 67, 67, 67, 67, 68,
        68, 68, 68, 68, 68, 68, 68, 68, 68, 68, 68, 68, 68, 68, 68, 68, 68, 68, 68,
        68, 68, 68, 68, 68, 68, 68, 68, 68, 68, 68, 68, 68, 68, 68, 68, 68, 68, 68,
        68, 68, 68, 68, 69, 69, 69, 69, 69, 69, 69, 69, 69, 69, 69, 69, 69, 69, 69,
        69, 69, 69, 69, 69, 69, 69, 69, 69, 69, 69, 69, 69, 69, 69, 69, 69, 69, 69,
        69, 69, 69, 69, 69, 69, 69, 69, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70,
        70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70,
        70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 70, 71, 71, 71, 71, 71, 71, 71, 71,
        71, 71, 71, 71, 71, 71, 71, 71, 71, 71, 71, 71, 71, 71, 71, 71, 71, 71, 71,
        71, 71, 71, 71, 71, 71, 71, 71, 71, 71, 71, 71, 71, 72, 72, 72, 72, 72, 72,
        72, 72, 72, 72, 72, 72, 72, 72, 72, 72, 72, 72, 72, 72, 72, 72, 72, 72, 72,
        72, 72, 72, 72, 72, 72, 72, 72, 72, 72, 72, 72, 72, 72, 73, 73, 73, 73, 73,
        73, 73, 73, 73, 73, 73, 73, 73, 73, 73, 73, 73, 73, 73, 73, 73, 73, 73, 73,
        73, 73, 73, 73, 73, 73, 73, 73, 73, 73, 73, 73, 73, 73, 74, 74, 74, 74, 74,
        74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 74,
        74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 74, 75, 75, 75, 75, 75, 75,
        75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75,
        75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 76, 76, 76, 76, 76, 76, 76, 76,
        76, 76, 76, 76, 76, 76, 76, 76, 76, 76, 76, 76, 76, 76, 76, 76, 76, 76, 76,
        76, 76, 76, 76, 76, 76, 76, 76, 77, 77, 77, 77, 77, 77, 77, 77, 77, 77, 77,
        77, 77, 77, 77, 77, 77, 77, 77, 77, 77, 77, 77, 77, 77, 77, 77, 77, 77, 77,
        77, 77, 77, 77, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78,
        78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 79,
        79, 79, 79, 79, 79, 79, 79, 79, 79, 79, 79, 79, 79, 79, 79, 79, 79, 79, 79,
        79, 79, 79, 79, 79, 79, 79, 79, 79, 79, 79, 79, 80, 80, 80, 80, 80, 80, 80,
        80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 80,
        80, 80, 80, 80, 80, 81, 81, 81, 81, 81, 81, 81, 81, 81, 81, 81, 81, 81, 81,
        81, 81, 81, 81, 81, 81, 81, 81, 81, 81, 81, 81, 81, 81, 81, 82, 82, 82, 82,
        82, 82, 82, 82, 82, 82, 82, 82, 82, 82, 82, 82, 82, 82, 82, 82, 82, 82, 82,
        82, 82, 82, 82, 82, 83, 83, 83, 83, 83, 83, 83, 83, 83, 83, 83, 83, 83, 83,
        83, 83, 83, 83, 83, 83, 83, 83, 83, 83, 83, 83, 83, 84, 84, 84, 84, 84, 84,
        84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84,
        84, 85, 85, 85, 85, 85, 85, 85, 85, 85, 85, 85, 85, 85, 85, 85, 85, 85, 85,
        85, 85, 85, 85, 85, 85, 85, 86, 86, 86, 86, 86, 86, 86, 86, 86, 86, 86, 86,
        86, 86, 86, 86, 86, 86, 86, 86, 86, 86, 86, 86, 87, 87, 87, 87, 87, 87, 87,
        87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 88, 88, 88,
        88, 88, 88, 88, 88, 88, 88, 88, 88, 88, 88, 88, 88, 88, 88, 88, 88, 88, 88,
        89, 89, 89, 89, 89, 89, 89, 89, 89, 89, 89, 89, 89, 89, 89, 89, 89, 89, 89,
        89, 89, 90, 90, 90, 90, 90, 90, 90, 90, 90, 90, 90, 90, 90, 90, 90, 90, 90,
        90, 90, 90, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91,
        91, 91, 91, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92,
        92, 92, 93, 93, 93, 93, 93, 93, 93, 93, 93, 93, 93, 93, 93, 93, 93, 93, 93,
        94, 94, 94, 94, 94, 94, 94, 94, 94, 94, 94, 94, 94, 94, 94, 94, 95, 95, 95,
        95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 96, 96, 96, 96, 96, 96, 96,
        96, 96, 96, 96, 96, 96, 96, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97,
        97, 98, 98, 98, 98, 98, 98, 98, 98, 98, 98, 98, 98, 99, 99, 99, 99, 99, 99,
        99, 99, 99, 99, 99, 100, 100, 100, 100, 100, 100, 100, 100, 100,
    ];
    let b = [
        3, 3, 5, 6, 7, 3, 6, 7, 3, 7, 3, 3, 9, 10, 11, 12, 13, 14, 15, 3, 10, 11,
        12, 13, 14, 15, 3, 11, 12, 13, 14, 15, 3, 12, 13, 14, 15, 3, 13, 14, 15, 3,
        14, 15, 3, 15, 3, 3, 5, 6, 7, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27,
        28, 29, 30, 31, 3, 5, 6, 7, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
        30, 31, 3, 5, 6, 7, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 3,
        5, 6, 7, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 3, 5, 6, 7, 21, 22,
        23, 24, 25, 26, 27, 28, 29, 30, 31, 3, 5, 6, 7, 22, 23, 24, 25, 26, 27, 28,
        29, 30, 31, 3, 5, 6, 7, 23, 24, 25, 26, 27, 28, 29, 30, 31, 3, 5, 6, 7, 24,
        25, 26, 27, 28, 29, 30, 31, 3, 5, 6, 7, 25, 26, 27, 28, 29, 30, 31, 3, 6,
        7, 26, 27, 28, 29, 30, 31, 3, 6, 7, 27, 28, 29, 30, 31, 3, 6, 7, 28, 29, 30,
        31, 3, 6, 7, 29, 30, 31, 3, 6, 7, 30, 31, 3, 6, 7, 31, 3, 6, 7, 3, 6, 7, 33,
        34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52,
        53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 3, 6, 7, 34, 35, 36, 37, 38, 39,
        40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58,
        59, 60, 61, 62, 63, 3, 6, 7, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46,
        47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 3, 6,
        7, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53,
        54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 3, 7, 37, 38, 39, 40, 41, 42, 43,
        44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62,
        63, 3, 7, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53,
        54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 3, 7, 39, 40, 41, 42, 43, 44, 45,
        46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 3,
        7, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57,
        58, 59, 60, 61, 62, 63, 3, 7, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51,
        52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 3, 7, 42, 43, 44, 45, 46,
        47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 3, 7,
        43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61,
        62, 63, 3, 7, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58,
        59, 60, 61, 62, 63, 3, 7, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56,
        57, 58, 59, 60, 61, 62, 63, 3, 7, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55,
        56, 57, 58, 59, 60, 61, 62, 63, 3, 7, 47, 48, 49, 50, 51, 52, 53, 54, 55,
        56, 57, 58, 59, 60, 61, 62, 63, 3, 7, 48, 49, 50, 51, 52, 53, 54, 55, 56,
        57, 58, 59, 60, 61, 62, 63, 3, 7, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58,
        59, 60, 61, 62, 63, 3, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62,
        63, 3, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 3, 52, 53, 54,
        55, 56, 57, 58, 59, 60, 61, 62, 63, 3, 53, 54, 55, 56, 57, 58, 59, 60, 61,
        62, 63, 3, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 3, 55, 56, 57, 58, 59,
        60, 61, 62, 63, 3, 56, 57, 58, 59, 60, 61, 62, 63, 3, 57, 58, 59, 60, 61,
        62, 63, 3, 58, 59, 60, 61, 62, 63, 3, 59, 60, 61, 62, 63, 3, 60, 61, 62, 63,
        3, 61, 62, 63, 3, 62, 63, 3, 63, 3, 3, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15,
        65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83,
        84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 3, 5,
        6, 7, 9, 10, 11, 12, 13, 14, 15, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76,
        77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95,
        96, 97, 98, 99, 100, 3, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15, 67, 68, 69, 70,
        71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89,
        90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 3, 5, 6, 7, 9, 10, 11, 12, 13,
        14, 15, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84,
        85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 3, 5, 6,
        7, 9, 10, 11, 12, 13, 14, 15, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79,
        80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98,
        99, 100, 3, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15, 70, 71, 72, 73, 74, 75, 76,
        77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95,
        96, 97, 98, 99, 100, 3, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15, 71, 72, 73, 74,
        75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93,
        94, 95, 96, 97, 98, 99, 100, 3, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15, 72, 73,
        74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92,
        93, 94, 95, 96, 97, 98, 99, 100, 3, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15, 73,
        74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92,
        93, 94, 95, 96, 97, 98, 99, 100, 3, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15, 74,
        75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93,
        94, 95, 96, 97, 98, 99, 100, 3, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15, 75, 76,
        77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95,
        96, 97, 98, 99, 100, 3, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15, 76, 77, 78, 79,
        80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98,
        99, 100, 3, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15, 77, 78, 79, 80, 81, 82, 83,
        84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 3, 5,
        6, 7, 9, 10, 11, 12, 13, 14, 15, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88,
        89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 3, 5, 6, 7, 9, 10, 11, 12,
        13, 14, 15, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94,
        95, 96, 97, 98, 99, 100, 3, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15, 80, 81, 82,
        83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 3,
        5, 6, 7, 9, 10, 11, 12, 13, 14, 15, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90,
        91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 3, 5, 6, 7, 10, 11, 12, 13, 14, 15,
        82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100,
        3, 5, 6, 7, 10, 11, 12, 13, 14, 15, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92,
        93, 94, 95, 96, 97, 98, 99, 100, 3, 5, 6, 7, 10, 11, 12, 13, 14, 15, 84, 85,
        86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 3, 5, 6, 7, 10,
        11, 12, 13, 14, 15, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98,
        99, 100, 3, 5, 6, 7, 10, 11, 12, 13, 14, 15, 86, 87, 88, 89, 90, 91, 92, 93,
        94, 95, 96, 97, 98, 99, 100, 3, 5, 6, 7, 10, 11, 12, 13, 14, 15, 87, 88, 89,
        90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 3, 5, 6, 7, 10, 11, 12, 13, 14,
        15, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 3, 5, 6, 7, 10, 11,
        12, 13, 14, 15, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 3, 5, 6,
        7, 10, 11, 12, 13, 14, 15, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 3,
        5, 6, 7, 10, 11, 12, 13, 14, 15, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100,
        3, 5, 6, 7, 10, 11, 12, 13, 14, 15, 92, 93, 94, 95, 96, 97, 98, 99, 100, 3,
        5, 6, 7, 10, 11, 12, 13, 14, 15, 93, 94, 95, 96, 97, 98, 99, 100, 3, 5, 6,
        7, 10, 11, 12, 13, 14, 15, 94, 95, 96, 97, 98, 99, 100, 3, 5, 6, 7, 10, 11,
        12, 13, 14, 15, 95, 96, 97, 98, 99, 100, 3, 5, 6, 7, 10, 11, 12, 13, 14, 15,
        96, 97, 98, 99, 100, 3, 5, 6, 7, 10, 11, 12, 13, 14, 15, 97, 98, 99, 100,
        3, 5, 6, 7, 10, 11, 12, 13, 14, 15, 98, 99, 100, 3, 5, 6, 7, 10, 11, 12, 13,
        14, 15, 99, 100, 3, 5, 6, 7, 10, 11, 12, 13, 14, 15, 100, 3, 5, 6, 7, 11,
        12, 13, 14, 15,
    ];
    let expected = [
        0, 1, 0, 0, 0, 1, 0, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0,
        0, 2, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 2, 0, 0, 0, 2, 0, 0, 2, 0, 2, 2, 1, 1,
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 1, 1, 1, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 2, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 1, 1, 1, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 2, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 1, 1, 1,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 2, 1, 1, 1,
        0, 0, 0, 0, 0, 0, 0, 2, 1, 1, 0, 0, 0, 0, 0, 0, 2, 1, 1, 0, 0, 0, 0, 0, 3,
        1, 1, 0, 0, 0, 0, 3, 1, 1, 0, 0, 0, 3, 1, 1, 0, 0, 3, 1, 1, 0, 3, 1, 1, 3,
        1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 3, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 1, 1, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 1,
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 3, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 3, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 3, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 3, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3,
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 1, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0,
        0, 0, 3, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 3, 0, 0, 0, 3, 0, 0, 3, 0, 3, 3, 2,
        2, 2, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 2, 2, 2, 1,
        1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 2, 2, 2, 1, 1, 1, 1, 1,
        1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 3, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 2,
        2, 2, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 2, 2, 2, 1, 1, 1, 1, 1, 1,
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 3, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 2, 2, 2,
        1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 2,
        2, 2, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 2, 2, 2,
        1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 3, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 2, 2, 2, 1, 1, 1, 1, 1, 1,
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 2,
        2, 2, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 3, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 2, 2, 2, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 2, 2, 2, 1, 1, 1, 1, 1, 1,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 2, 2, 2, 1, 1, 1,
        1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 2, 2, 2, 1,
        1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 2, 2, 2,
        1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 2, 2, 2,
        1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 2, 2, 2, 1,
        1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 2, 2, 2, 1, 1, 1,
        1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 2, 2, 2, 1, 1, 1, 1, 1, 1,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 2, 2, 2, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 4, 2, 2, 2, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        4, 2, 2, 2, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 4, 2, 2, 2, 1, 1, 1,
        1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 4, 2, 2, 2, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0,
        0, 4, 2, 2, 2, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 4, 2, 2, 2, 1, 1, 1, 1, 1,
        1, 0, 0, 0, 0, 4, 2, 2, 2, 1, 1, 1, 1, 1, 1, 0, 0, 0, 4, 2, 2, 2, 1, 1, 1,
        1, 1, 1, 0, 0, 4, 2, 2, 2, 1, 1, 1, 1, 1, 1, 0, 4, 2, 2, 2, 1, 1, 1, 1, 1,
    ];

    let mut i = 0;

    while i < 1825 {
        let ai: u64 = a[i];
        let bi: u64 = b[i];
        let expected_val: u64 = expected[i];

        let result = U128::from(ai).log(U128::from(bi));
        assert_eq(result, U128::from(expected_val));
        i += 1;
    }

    assert(prior_flags == flags());
}

#[test]
fn u128_overflowing_add() {
    let prior_flags = disable_panic_on_overflow();
    let a = U128::max();
    let b = U128::from((0, 1));
    let c = a + b;

    assert(c == U128::from((0, 0)));

    set_flags(prior_flags);
}

#[test]
fn u128_underflowing_sub() {
    let prior_flags = disable_panic_on_overflow();
    let a = U128::from((0, 1));
    let b = U128::from((0, 2));
    let c = a - b;

    assert(c == U128::max());

    set_flags(prior_flags);
}

#[test]
fn u128_overflowing_mul() {
    let prior_flags = disable_panic_on_overflow();
    let a = U128::max();
    let b = U128::from((0, 2));
    let c = a * b;

    // 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFE
    assert(c == U128::from((0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFE)));

    set_flags(prior_flags);
}

#[test]
fn u128_overflowing_pow() {
    // Overflow on pow should return 0 if panic is disabled
    let prior_flags = disable_panic_on_overflow();
    let a = U128::max();

    let res = a.pow(2);

    assert(res == U128::from((0, 0)));

    assert(U128::from(u64::max()).pow(100) == U128::zero());

    assert(U128::from(2u32).pow(150) == U128::zero());

    let lower_max = U128::from((0, u64::max()));
    let with_upper_1 = lower_max + U128::from(1u32);
    assert(with_upper_1 == U128::from((1, 0)));
    assert(with_upper_1 > lower_max);
    let powered_to_zero = with_upper_1.pow(2);
    assert(powered_to_zero == U128::zero());


    let u128_upper_and_lower_not_zero = U128::from((1, 1));
    let upper_and_lower_result = u128_upper_and_lower_not_zero.pow(2);
    assert(upper_and_lower_result == U128::zero());

    set_flags(prior_flags);
}

#[test]
fn u128_unsafemath_log2() {
    let prior_flags = disable_panic_on_unsafe_math();
    // 0 is not a valid operand for log2
    let a = U128::zero();
    let res = a.log2();

    assert(res == U128::zero());

    set_flags(prior_flags);
}

#[test]
fn u128_as_u256() {
    let mut vals = Vec::new();
    vals.push(0);
    vals.push(1);
    vals.push(2);
    vals.push(u64::max() - 1);
    vals.push(u64::max());

    for val in vals.iter() {
        // Ensure parity with u256::from(val)
        let u128_val = U128::from(val);
        let u256_val = u128_val.as_u256();
        assert(u256_val == u256::from(val));

        // Ensure parity with transmute u256 conversion
        let trm_val = __transmute::<(u64, u64, u64, u64), u256>((0, 0, 0, val));
        assert(u256_val == trm_val);

        for val2 in vals.iter() {
            // Ensure parity with u256::from(0, 0, val, val2)
            let u128_val = U128::from((val, val2));
            let u256_val = u128_val.as_u256();
            assert(u256_val == u256::from((0, 0, val, val2)));

            // Ensure parity with transmute u256 conversion
            let trm_val = __transmute::<(u64, u64, u64, u64)>((0, 0, val, val2));
            assert(u256_val == trm_val);
        }
    }
}
