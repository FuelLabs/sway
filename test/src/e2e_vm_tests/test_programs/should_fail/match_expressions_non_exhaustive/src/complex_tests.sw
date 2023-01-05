library complex_tests;

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

const ORACLE_1: u64 = 2;
const ORACLE_2: u64 = 3;
const ORACLE_3: u64 = 4;

struct MyAddress {
    inner: u64,
}

struct MyContract {
    inner: bool
}

enum MyIdentity {
    Address: MyAddress,
    Contract: MyContract,
}

enum MyResult<T, E> {
    Ok: T,
    Err: E,
}

enum MyAuthError {
    ReasonOne: (),
    ReasonTwo: (),
}

pub fn enum_match_exp_bugfix_test() {
    let sender: MyResult<MyIdentity, MyAuthError> = MyResult::Ok(MyIdentity::Address(MyAddress { inner: 7 }));
    let res = match sender {
        MyResult::Ok(MyIdentity::Address(MyAddress { inner: ORACLE_1 })) => 1,
        MyResult::Ok(MyIdentity::Address(MyAddress { inner: ORACLE_2 })) => 2,
        MyResult::Ok(MyIdentity::Address(MyAddress { inner: ORACLE_3 })) => 3,
        MyResult::Err(_) => 5,
    };
    assert(res == 4);
}
