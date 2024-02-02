script;

use std::bytes::Bytes;
use std::b512::B512;

fn main() -> bool {

    // test_assert_ne_u64
    let a = 42;
    let b = 40;
    assert_ne(a, b);

    // test_assert_ne_u32
    let c = 42u32;
    let d = 40u32;
    assert_ne(c, d);

    // test_assert_ne_u16
    let e = 42u16;
    let f = 40u16;
    assert_ne(e, f);

    // test_assert_ne_u8
    let g = 42u8;
    let h = 40u8;
    assert_ne(g, h);

    // test_assert_ne_b256
    let i: b256 = 0b0000000000000000000000000000000000000000000000000000000000000001_0000000000000000000000000000000000000000000000000000000000000001_0000000000000000000000000000000000000000000000000000000000000001_0000000000000000000000000000000000000000000000000000000000000010;
    let j: b256 = 0b1000000000000000000000000000000000000000000000000000000000000000_1000000000000000000000000000000000000000000000000000000000000000_1000000000000000000000000000000000000000000000000000000000000000_1000000000000000000000000000000000000000000000000000000000000001;
    assert_ne(i, j);

    // test_assert_ne_address
    let value = 0xBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEF;
    let value2 = 0xBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEE;
    let k = Address::from(value);
    let l = Address::from(value2);
    assert_ne(k, l);

    // test_assert_ne_contract_id
    let m = ContractId::from(value);
    let n = ContractId::from(value2);
    assert_ne(m, n);

    // test_assert_ne_bytes
    let mut q = Bytes::new();
    let mut r = Bytes::new();
    q.push(42u8);
    q.push(11u8);
    q.push(69u8);
    r.push(42u8);
    r.push(11u8);
    r.push(70u8);
    assert_ne(q, r);

    // test_assert_ne_b512
    let value = 0xBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEF;
    let value2 = 0xBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEE;
    let s = B512::from((value, value));
    let t = B512::from((value2, value2));
    assert_ne(s, t);

    // test_assert_ne_identity
    let value = 0xBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEF;
    let value2 = 0xBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEE;
    let u = Identity::Address(Address::from(value));
    let v = Identity::Address(Address::from(value2));
    let w = Identity::ContractId(ContractId::from(value));
    let x = Identity::ContractId(ContractId::from(value2));
    assert_ne(u, v);
    assert_ne(w, x);

    true
}
