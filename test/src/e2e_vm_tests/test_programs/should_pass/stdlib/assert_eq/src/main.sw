script;

use std::assert::assert_eq;
use std::address::Address;
use std::contract_id::ContractId;
use std::bytes::Bytes;
use std::vec::Vec;
use std::identity::Identity;
use std::b512::B512;
use std::logging::log;

fn main() -> bool {

// test_assert_eq_u64
    let a = 42;
    let b = 40 + 2;
    log(assert_eq::<u64>(a, b));

// // test_assert_eq_u32
//     let c = 42u32;
//     let d = 40u32 + 2u32;
//     assert_eq(c, d);

// // test_assert_eq_u16
//     let e = 42u16;
//     let f = 40u16 + 2u16;
//     assert_eq(e, f);

// // test_assert_eq_u8
//     let g = 42u8;
//     let h = 40u8 + 2u8;
//     assert_eq(g, h);

// // test_assert_eq_b256
//     let i: b256 = 0b0000000000000000000000000000000000000000000000000000000000000001_0000000000000000000000000000000000000000000000000000000000000001_0000000000000000000000000000000000000000000000000000000000000001_0000000000000000000000000000000000000000000000000000000000000010;
//     let j: b256 = 0b1000000000000000000000000000000000000000000000000000000000000000_1000000000000000000000000000000000000000000000000000000000000000_1000000000000000000000000000000000000000000000000000000000000000_1000000000000000000000000000000000000000000000000000000000000001 << 1;
//     assert_eq(i, j);

// // test_assert_eq_address
//     let value = 0xBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEF;
//     let k = Address::from(value);
//     let l = Address::from(value);
//     assert_eq(k, l);

// // test_assert_eq_contract_id
//     let m = ContractId::from(value);
//     let n = ContractId::from(value);
//     assert_eq(m, n);

// // test_assert_eq_bytes
//     let mut q = Bytes::new();
//     let mut r = Bytes::new();
//     q.push(42);
//     q.push(11);
//     q.push(69);
//     r.push(42);
//     r.push(11);
//     r.push(69);
//     assert_eq(q, r);

// // test_assert_eq_b512
//     let value = 0xBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEF;
//     let s = B512::from((value, value));
//     let t = B512::from((value, value));
//     assert_eq(s, t);

// // test_assert_eq_identity
//     let value = 0xBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEFBEEF;
//     let u = Identity::Address(Address::from(value));
//     let v = Identity::Address(Address::from(value));
//     let w = Identity::ContractId(ContractId::from(value));
//     let x = Identity::ContractId(ContractId::from(value));
//     assert_eq(u, v);
//     assert_eq(w, x);

    true
}
