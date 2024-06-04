library;

use std::u128::U128;

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
}

#[test(should_revert)]
fn revert_u128_add() {
    let one = U128::from((0, 1));
    let max_u64 = U128::from((u64::max(), u64::max()));

    let _result = one + max_u64;
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
}

#[test(should_revert)]
fn revert_u128_multiply() {
    let first = U128::from((0, 2));
    let second = U128::from((u64::max(), 1));

    let _result = first * second;
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
    let u_128_21: U128 = U128::from((0, 21));
    let u_128_42: U128 = U128::from((0, 42));
    let u_128_64: U128 = U128::from((0, 64));
    let u_128_100: U128 = U128::from((0, 100));
    let u_128_127: U128 = U128::from((0, 127));
    let u_128_max_div_2: U128 = U128::from((1, 0));
    let u_128_max: U128 = U128::max();

    assert(u_128_2.log(u_128_2) == u_128_1);
    assert(u_128_1.log(u_128_3) == u_128_0);
    assert(u_128_8.log(u_128_2) == u_128_3);
    assert(u_128_100.log(u_128_10) == u_128_2);
    assert(u_128_100.log(u_128_2) == u_128_6);
    assert(u_128_100.log(u_128_9) == u_128_2);
    assert(u_128_max.log(u_128_2) == u_128_127);
    assert(u_128_max.log(u_128_9) == u_128_42);
    assert(u_128_max_div_2.log(u_128_2) == u_128_64);
    assert(u_128_max_div_2.log(u_128_9) == u_128_21);
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
