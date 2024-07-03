library; 

// returns 3
fn literals() -> u256 {
    0x0000000000000000000000000000000000000000000000000000000000000001u256 + 0x0000000000000000000000000000000000000000000000000000000000000002u256
}

// returns 1
fn locals() -> u256 {
    let a = 0x0000000000000000000000000000000000000000000000000000000000000005u256;
    let b = 0x0000000000000000000000000000000000000000000000000000000000000004u256;
    let c = 0x0000000000000000000000000000000000000000000000000000000000000001u256;
    let d = 0x0000000000000000000000000000000000000000000000000000000000000004u256;
    let e = 0x0000000000000000000000000000000000000000000000000000000000000002u256;
    
    let result = ((a + b - c) * d / e) % a;
    assert(result == 0x0000000000000000000000000000000000000000000000000000000000000001u256);

    result
}

// returns 11
fn bitwise_operators() -> u256 {
    let a = 18446744073709551615u64;
    let b = 3u64;
    let c = 2u64;
    let d = 4u64;
    let e = 15u64;

    let r = !(a - b) & c | d ^ e;
    assert(r == 11);

    let a = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu256;
    let b = 0x0000000000000000000000000000000000000000000000000000000000000003u256;
    let c = 0x0000000000000000000000000000000000000000000000000000000000000002u256;
    let d = 0x0000000000000000000000000000000000000000000000000000000000000004u256;
    let e = 0x000000000000000000000000000000000000000000000000000000000000000Fu256;
    let r = !(a - b) & c | d ^ e;

    assert(r == 0x000000000000000000000000000000000000000000000000000000000000000Bu256);

    r
}

// returns 8
fn shift_operators() -> u256 {
    // assert u256 shifts behave like u64 shifts
    let a = 0b10;
    assert(a >> 1 == 1);
    assert(a >> 2 == 0);
    assert(a >> 3 == 0);

    let a = 0b1000000000000000000000000000000000000000000000000000000000000000;
    assert(a >> 1 == 0b0100000000000000000000000000000000000000000000000000000000000000);

    let a = 0b0100000000000000000000000000000000000000000000000000000000000000;
    assert(a << 1 == 0b1000000000000000000000000000000000000000000000000000000000000000);
    assert(a << 2 == 0);
    assert(a << 3 == 0);

    assert(0b0000000000000000000000000000000000000000000000000000000000000001 << 63 == 0b1000000000000000000000000000000000000000000000000000000000000000);
    assert(0b1000000000000000000000000000000000000000000000000000000000000000 >> 63 == 0b0000000000000000000000000000000000000000000000000000000000000001);

    // now u256
    let a = 0x0000000000000000000000000000000000000000000000000000000000000002u256;
    assert(a >> 1 == 0x0000000000000000000000000000000000000000000000000000000000000001u256);
    assert(a >> 2 == 0x0000000000000000000000000000000000000000000000000000000000000000u256);
    assert(a >> 3 == 0x0000000000000000000000000000000000000000000000000000000000000000u256);
    let a = 0x8000000000000000000000000000000000000000000000000000000000000000u256;
    assert(a >> 1 == 0x4000000000000000000000000000000000000000000000000000000000000000u256);

    let a = 0x4000000000000000000000000000000000000000000000000000000000000000u256;
    assert(a << 1 == 0x8000000000000000000000000000000000000000000000000000000000000000u256);
    assert(a << 2 == 0x0000000000000000000000000000000000000000000000000000000000000000u256);
    assert(a << 3 == 0x0000000000000000000000000000000000000000000000000000000000000000u256);

    assert(0x0000000000000000000000000000000000000000000000000000000000000001u256 << 255 == 0x8000000000000000000000000000000000000000000000000000000000000000u256);
    assert(0x8000000000000000000000000000000000000000000000000000000000000000u256 >> 255 == 0x0000000000000000000000000000000000000000000000000000000000000001u256);

    // return some value
    let a = 0x0000000000000000000000000000000000000000000000000000000000000002u256;
    (a << 4) >> 2
}

// returns 0
fn comparison_operators() -> u256 {
    let a = 0x0000000000000000000000000000000000000000000000000000000000000001u256;
    let b = 0x0000000000000000000000000000000000000000000000000000000000000002u256;
    let c = 0x0000000000000000000000000000000000000000000000000000000000000003u256;
    let d = 0x0000000000000000000000000000000000000000000000000000000000000003u256;
    
    assert(c == c);
    assert(c <= c);
    assert(c >= c);

    assert(c == d);
    assert(d == c);
    assert(c <= d);
    assert(c >= d);
    assert(d <= c);
    assert(d >= c);

    assert(a < b);
    assert(b < c);
    assert(a < c);

    assert(a <= b);
    assert(b <= c);
    assert(a <= c);

    assert(b > a);
    assert(c > b);
    assert(c > a);

    return 0x0000000000000000000000000000000000000000000000000000000000000000u256;
}

#[test]
fn should_be_able_to_use_literals() {
    let result = 0x0000000000000000000000000000000000000000000000000000000000000003u256;
    assert_eq(literals(), result);
}

#[test]
fn should_be_able_to_use_locals() {
    let result = 0x0000000000000000000000000000000000000000000000000000000000000001u256;
    assert_eq(locals(), result);
}

#[test]
fn should_be_able_to_use_bitwise_operators() {
    let result = 0x000000000000000000000000000000000000000000000000000000000000000Bu256;
    assert_eq(bitwise_operators(), result);
}

#[test]
fn should_be_able_to_use_shift_operators() {
    let result = 0x0000000000000000000000000000000000000000000000000000000000000008u256;
    assert_eq(shift_operators(), result);
}

#[test]
fn should_be_able_to_use_comparison_operators() {
    let result = 0x0000000000000000000000000000000000000000000000000000000000000000u256;
    assert_eq(comparison_operators(), result);
}

#[test(should_revert)]
fn should_revert_on_overflow() -> u256 {
    let a = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu256;
    a + 0x0000000000000000000000000000000000000000000000000000000000000001u256
}

#[test(should_revert)]
fn should_revert_on_underflow() -> u256 {
    let a = 0x0000000000000000000000000000000000000000000000000000000000000000u256;
    a - 0x0000000000000000000000000000000000000000000000000000000000000001u256
}

#[test(should_revert)]
fn should_revert_on_div_zero() -> u256 {
    let a = 0x0000000000000000000000000000000000000000000000000000000000000000u256;
    a / 0x0000000000000000000000000000000000000000000000000000000000000000u256
}

#[test]
fn type_inference_numeric_1() -> u256 {
    let a = 1;
    let b = 0x2u256;
    b * a
}

#[test]
fn type_inference_numeric_2() -> u256 {
   let mut result = 0;
   result = 3.as_u256();
   result
}

#[test]
fn incorrect_def_modeling() -> u256 {
    let c: u256 = 1;
    c % c  // this emits a WQAM instruction
}
