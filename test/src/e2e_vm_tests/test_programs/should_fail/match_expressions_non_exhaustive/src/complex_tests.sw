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

// const ORACLE_1: u64 = 2;
// const ORACLE_2: u64 = 3;
// const ORACLE_3: u64 = 4;

// fn enum_match_exp_bugfix_test() {
//     let sender: Result<Identity, AuthError> = msg_sender();
//     match sender {
//         Result::Ok(Identity::Address(Address { value: ORACLE_1 })) => true,
//         Result::Ok(Identity::Address(Address { value: ORACLE_2 })) => true,
//         Result::Ok(Identity::Address(Address { value: ORACLE_3 })) => true,
//         // Result::Ok(_) => false,
//         Result::Err(_) => false,
//     }
// }
