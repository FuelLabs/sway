library;

// This file tests the reported errors and warnings in various instances when a 'panic'
// occurs in a non-statement position. This is allowed, but will often result in
// unreachable code or similar warning situations.

struct S { x : u64, y : u64, }
enum Enum {
    A: (u64, u64),
}
// Single-variant enums are treated differently in the IR generation, so a second test
// enum is necessary.
enum EnumMultivariant {
    A: (),
    B: (u64, u64),
}

#[error_type]
enum E {
    #[error(m = "Error description.")]
    E: u64,
}

// Legal panic expressions. These should warn of unreachable code, but otherwise no error.

#[test(should_revert)]
fn in_init() -> u64 {
    let _ = panic E::E(42);
    
    045
}

#[test(should_revert)]
fn in_array() -> u64 {
    let _ = [panic E::E(42), panic E::E(43)];
    
    1450
}

// Arrays of length 1 are treated differently
#[test(should_revert)]
fn in_length_1_array() -> u64 {
    let _ = [panic E::E(42)];
    
    1451
}

// The first element of an array is treated differently
#[test(should_revert)]
fn in_length_2_array_first() -> u64 {
    let _ = [panic E::E(42), 0];
    
    1452
}

// The first element of an array is treated differently
#[test(should_revert)]
fn in_length_2_array_second() -> u64 {
    let _ = [0, panic E::E(42)];
    
    1453
}

#[test(should_revert)]
fn in_tuple() -> u64 {
    let _ = (panic E::E(42), panic E::E(43));
    
    245
}

#[test(should_revert)]
fn in_struct() -> u64 {
    let _ = S { x: panic E::E(42), y: panic E::E(43) };
    
    345
}

#[test(should_revert)]
fn in_parentheses() -> u64 {
    let _ = (panic E::E(42));

    445
}

#[test(should_revert)]
fn in_if_condition() -> u64 {
    let _ = if panic E::E(42) {
        543
    }
    else {
        345
    };
    
    645
}

#[test(should_revert)]
fn in_while_condition() -> u64 {
    while panic E::E(42) {
        break;
    };
    
    745
}

#[test(should_revert)]
fn in_enum() -> u64 {
    let _ = Enum::A((panic E::E(42), panic E::E(43)));
    
    845
}

#[test(should_revert)]
fn in_enum_multivariant() -> u64 {
    let _ = EnumMultivariant::B((panic E::E(42), panic E::E(43)));
    
    945
}

fn helper_fun(x : u64, y : u64) -> u64 {
    x + y
}

#[test(should_revert)]
fn in_fun_arg() -> u64 {
    let _ = helper_fun(panic E::E(42), panic E::E(43));

    1045
}

#[test(should_revert)]
fn in_lazy_and() -> u64 {
    let _ = (panic E::E(42)) && panic E::E(43);

    1145
}

#[test(should_revert)]
fn in_lazy_or() -> u64 {
    let _ = (panic E::E(42)) || panic E::E(43);

    1245
}

#[test(should_revert)]
fn in_match_scrutinee() -> u64 {
    let _ = match panic E::E(42) {
        _ => 5411,
    };

    1345
}
