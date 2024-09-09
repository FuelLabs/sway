library;

#[test]
fn assert_assert() {
    assert(true);
    assert(1 == 1);
    assert(1 + 1 == 2);
    assert(!false);
    assert(true && true);
    assert(true || false);
    assert(!false && !false);
}

#[test(should_revert)]
fn revert_assert_assert_when_not_true() {
    assert(false);
}

#[test]
fn assert_assert_eq() {
    use std::bytes::Bytes;

    // assert_eq u64
    let a = 42;
    let b = 40 + 2;
    assert_eq(a, b);

    // assert_eq u32
    let c = 42u32;
    let d = 40u32 + 2u32;
    assert_eq(c, d);

    // assert_eq u16
    let e = 42u16;
    let f = 40u16 + 2u16;
    assert_eq(e, f);

    // assert_eq u8
    let g = 42u8;
    let h = 40u8 + 2u8;
    assert_eq(g, h);

    // assert_eq b256
    let i: b256 = 0b0000000000000000000000000000000000000000000000000000000000000001_0000000000000000000000000000000000000000000000000000000000000001_0000000000000000000000000000000000000000000000000000000000000001_0000000000000000000000000000000000000000000000000000000000000010;
    let j: b256 = 0b1000000000000000000000000000000000000000000000000000000000000000_1000000000000000000000000000000000000000000000000000000000000000_1000000000000000000000000000000000000000000000000000000000000000_1000000000000000000000000000000000000000000000000000000000000001 << 1;
    assert_eq(i, j);

    // assert_eq u256
    let k: u256 = 0x02u256;
    let l: u256 = 0x01u256 + 0x01u256;
    assert_eq(k, l);

    // assert_eq struct
    let value = 0xBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEF;
    let m = Address::from(value);
    let n = Address::from(value);
    assert_eq(m, n);

    // assert_eq heap
    let mut o = Bytes::new();
    let mut p = Bytes::new();
    o.push(42u8);
    o.push(11u8);
    o.push(69u8);
    p.push(42u8);
    p.push(11u8);
    p.push(69u8);
    assert_eq(o, p);

    // TODO: Uncomment when https://github.com/FuelLabs/sway/issues/6086 is resolved.
    // assert_eq array
    // let value = 0xBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEF;
    // let mut q: [u64; 3] = [0, 0, 0];
    // let mut r: [u64; 3] = [0, 0, 0];
    // q[0] = 1;
    // q[1] = 2;
    // q[2] = 3;
    // r[0] = 1;
    // r[1] = 2;
    // r[2] = 3;
    // assert_eq(q, r);

    // assert_eq enum
    let value = 0xBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEF;
    let s = Identity::Address(Address::from(value));
    let t = Identity::Address(Address::from(value));
    let u = Identity::ContractId(ContractId::from(value));
    let v = Identity::ContractId(ContractId::from(value));
    assert_eq(s, t);
    assert_eq(u, v);
}

#[test(should_revert)]
fn revert_assert_assert_eq() {
    assert_eq(1, 2);
}

#[test]
fn assert_assert_ne() {
    use std::bytes::Bytes;

    // assert_ne u64
    let a = 42;
    let b = 40;
    assert_ne(a, b);

    // assert_ne u32
    let c = 42u32;
    let d = 40u32;
    assert_ne(c, d);

    // assert_ne u16
    let e = 42u16;
    let f = 40u16;
    assert_ne(e, f);

    // assert_ne u8
    let g = 42u8;
    let h = 40u8;
    assert_ne(g, h);

    // assert_ne b256
    let i: b256 = 0b0000000000000000000000000000000000000000000000000000000000000001_0000000000000000000000000000000000000000000000000000000000000001_0000000000000000000000000000000000000000000000000000000000000001_0000000000000000000000000000000000000000000000000000000000000010;
    let j: b256 = 0b1000000000000000000000000000000000000000000000000000000000000000_1000000000000000000000000000000000000000000000000000000000000000_1000000000000000000000000000000000000000000000000000000000000000_1000000000000000000000000000000000000000000000000000000000000001;
    assert_ne(i, j);

    // assert_ne u256
    let k: u256 = 0x02u256;
    let l: u256 = 0x01u256;
    assert_ne(k, l);

    // assert_ne struct
    let value = 0xBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEF;
    let value2 = 0xBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEE;
    let m = Address::from(value);
    let n = Address::from(value2);
    assert_ne(m, n);

    // test_assert_ne heap
    let mut o = Bytes::new();
    let mut p = Bytes::new();
    o.push(42u8);
    o.push(11u8);
    o.push(69u8);
    p.push(42u8);
    p.push(11u8);
    p.push(70u8);
    assert_ne(o, p);

    // TODO: Uncomment when https://github.com/FuelLabs/sway/issues/6086 is resolved.
    // assert_ne array
    // let mut q: [u64; 3] = [1, 2, 3];
    // let mut r: [u64; 3] = [0, 0, 0];
    // assert_eq(q, r);

    // assert_ne enum
    let value = 0xBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEF;
    let s = Identity::Address(Address::from(value));
    let t = Identity::Address(Address::from(value));
    let u = Identity::ContractId(ContractId::from(value));
    let v = Identity::ContractId(ContractId::from(value));
    assert_ne(u, t);
    assert_ne(s, v);
    assert_ne(s, u);
    assert_ne(v, t);
}

#[test(should_revert)]
fn revert_assert_assert_ne() {
    assert_ne(1, 1);
}
