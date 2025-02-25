library;

const ZERO: b256 = b256::zero();
const ONE: b256 = 0x0000000000000000000000000000000000000000000000000000000000000001;
const TWO: b256 = 0x0000000000000000000000000000000000000000000000000000000000000002;
const THREE: b256 = 0x0000000000000000000000000000000000000000000000000000000000000003;
const FOUR: b256 = 0x0000000000000000000000000000000000000000000000000000000000000004;
const FIVE: b256 = 0x0000000000000000000000000000000000000000000000000000000000000005;
const NINE: b256 = 0x0000000000000000000000000000000000000000000000000000000000000009;
const TEN: b256 = 0x000000000000000000000000000000000000000000000000000000000000000A;

#[test]
fn wqml() {
    let mut res = ZERO;

    let a: b256 = TWO;
    let b: b256 = TWO;

    asm(res: res, a: a, b: b) {
        wqml res a b i48;
    }

    assert_eq(res, FOUR);

    let a: u64 = 5;
    let b: u64 = 2;

    asm(res: res, a: a, b: b) {
        wqml res a b i0;
    }

    assert_eq(res, TEN);

    asm(res: res, a: ZERO, b: ZERO) {
        wqml res a b i48;
    }

    assert_eq(res, ZERO);

    let a: b256 = THREE;
    let b: u64 = 3;

    asm(res: res, a: a, b: b) {
        wqml res a b i16;
    }

    assert_eq(res, NINE);
}

#[test]
fn all_in_one() {
    // WQCM
    let a: b256 = TWO;
    let b: b256 = TWO;

    let bool_res = asm(b_res, a: a, b: b) {
        wqcm b_res a b i32; // a == b
        b_res: bool
    };

    assert_eq(bool_res, true);

    // WQOP
    let mut res = TEN;

    let a: b256 = TEN;
    let b: b256 = NINE;

    asm(res: res, a: a, b: b) {
        wqop res a b i33; // a - b
    };

    assert_eq(res, ONE);

    // WQML
    let mut res = ZERO;

    let a: b256 = TWO;
    let b: b256 = TWO;

    asm(res: res, a: a, b: b) {
        wqml res a b i48; // 2 * 2
    }

    assert_eq(res, FOUR);

    // WQDV
    let mut res = ZERO;

    let a: b256 = TEN;
    let b: b256 = FIVE;

    asm(res: res, a: a, b: b) {
        wqdv res a b i32; // 10 / 5
    }

    assert_eq(res, TWO);

    // WQMD
    let mut res = ZERO;

    let a: b256 = TEN;
    let b: b256 = TWO;
    let c: b256 = FIVE;

    asm(res: res, a: a, b: b, c: c) {
        wqmd res a b c; // (10 * 2) / 5
    }

    assert_eq(res, FOUR);

    // WQAM
    let mut res = ZERO;

    let a: b256 = TEN;
    let b: b256 = TWO;
    let c: b256 = FIVE;

    asm(res: res, a: a, b: b, c: c) {
        wqam res a b c; // (10 + 2) % 5
    }

    assert_eq(res, TWO);

    // WQMM
    let mut res = ZERO;

    let a: b256 = TEN;
    let b: b256 = TWO;
    let c: b256 = THREE;

    asm(res: res, a: a, b: b, c: c) {
        wqmm res a b c; // (10 * 2) % 3
    }

    assert_eq(res, TWO);
}
