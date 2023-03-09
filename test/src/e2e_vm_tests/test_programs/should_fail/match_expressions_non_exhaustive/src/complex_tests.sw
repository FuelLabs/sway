library;

use std::result::*;

fn a(x: u64) -> u64 {
    match x {
        7 => { 0 },
        _ => { 1 },
    }
}

fn b(x: u64) -> u64 {
    match x {
        14 => { 7 },
        _ => { 1 },
    }
}

fn c(x: u64) -> u64 {
    match x {
        21 => { 7 },
        _ => { 1 },
    }
}

pub fn nested_match_tests() {
    // should succeed
    let foo = match (match 1 {
            1 => { 1 },
            _ => { 0 },
        }) {
        0 => { 42 },
        _ => { 0 },
    };
    assert(foo == 0);

    // should succeed
    let q = 21;
    let foo = match a(match q {
        14 => { b(q) },
        21 => { c(q) },
        _ => { q },
    }) {
        0 => { 42 },
        _ => { 24 },
    };
    assert(foo == 42);
}

const ORACLE_1: u64 = 1;
const ORACLE_3: u64 = 3;

struct MyAddress {
    inner: u64,
}

struct MyContractId {
    inner: bool
}

enum MyIdentity {
    Address: MyAddress,
    ContractId: MyContractId,
}

enum MyAuthError {
    ReasonOne: (),
    ReasonTwo: (),
}

pub fn enum_match_exp_bugfix_test() {
    let a: Result<MyIdentity, MyAuthError> = Result::Ok(
        MyIdentity::Address(
            MyAddress { inner: 7 }
        )
    );

    // should fail with non-exhaustive
    let b = match a {
        Result::Ok(MyIdentity::Address(_)) => 1,
        // missing Result::Ok(MyIdentity::ContractId(_))
        Result::Err(_) => 5,
    };

    // should succeed
    let c = match a {
        Result::Ok(MyIdentity::Address(_)) => 1,
        Result::Ok(MyIdentity::ContractId(_)) => 4,
        Result::Err(_) => 5,
    };
    assert(c == 4);

    // should fail with non-exhaustive
    let d = match a {
        Result::Ok(MyIdentity::Address(MyAddress { inner: ORACLE_1 })) => 1,
        // missing Result::Ok(MyIdentity::Address(MyAddress { inner: 0, 2..MAX }))
        Result::Ok(MyIdentity::ContractId(_)) => 4,
        Result::Err(_) => 5,
    };

    // should fail with non-exhaustive
    let e = match a {
        Result::Ok(MyIdentity::Address(MyAddress { inner: ORACLE_1 })) => 1,
        // missing Result::Ok(MyIdentity::ContractId(_))
        Result::Err(_) => 5,
    };

    // should fail with non-exhaustive
    let f = match a {
        Result::Ok(MyIdentity::Address(MyAddress { inner: ORACLE_1 })) => 1,
        Result::Ok(MyIdentity::ContractId(_)) => 2,
        // missing Result::Ok(MyIdentity::Address(MyAddress { inner: 0, 2..MAX }))
        Result::Err(_) => 5,
    };

    // should fail with non-exhaustive
    let g = match a {
        Result::Ok(MyIdentity::Address(MyAddress { inner: ORACLE_1 })) => 1,
        Result::Ok(MyIdentity::ContractId(_)) => 2,
        Result::Ok(MyIdentity::Address(MyAddress { inner: ORACLE_3 })) => 3,
        // missing Result::Ok(MyIdentity::Address(MyAddress { inner: 0, 2, 4..MAX }))
        Result::Err(_) => 5,
    };
}

pub fn enum_match_exp_bugfix_test2() {
    let a: Result<MyIdentity, MyAuthError> = Result::Ok(
        MyIdentity::ContractId(
            MyContractId { inner: false }
        )
    );

    // should succeed
    let b = match a.unwrap() {
        MyIdentity::Address(_) => 1,
        MyIdentity::ContractId(_) => 2,
    };
    assert(b == 2);
}
