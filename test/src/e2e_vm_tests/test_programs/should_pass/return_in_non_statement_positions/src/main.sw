script;

// This file tests the reported errors and warnings in various instances when a 'return'
// occurs in a non-statement position. This is allowed, but will often result in
// unreachable code or similar warning situations.

pub struct S { x : u64, y : u64, }
pub enum Enum {
    A: (u64, u64),
}
// Single-variant enums are treated differently in the IR generation, so a second test
// enum is necessary.
pub enum EnumMultivariant {
    A: (),
    B: (u64, u64),
}

// Legal return types. These should warn of unreachable code, but otherwise no error.

fn in_init() -> u64 {
    let _ = return 42;
    
    045
}

fn in_array() -> u64 {
    let _ = [return 42, return 43];
    
    1450
}

// Arrays of length 1 are treated differently
fn in_length_1_array() -> u64 {
    let _ = [return 42];
    
    1451
}

// The first element of an array is treated differently
fn in_length_2_array_first() -> u64 {
    let _ = [return 42, 0];
    
    1452
}

// The first element of an array is treated differently
fn in_length_2_array_second() -> u64 {
    let _ = [0, return 42];
    
    1453
}

fn in_tuple() -> u64 {
    let _ = (return 42, return 43);
    
    245
}

fn in_struct() -> u64 {
    let _ = S { x: return 42, y: return 43 };
    
    345
}

fn in_parentheses() -> u64 {
    let _ = (return 42);

    445
}

fn in_arithmetic() -> u64 {
    let _ = return 43 + return 42; // Grouped as return (43 + return 42), so returns 42

    545
}

fn in_if_condition() -> u64 {
    let _ = if return 42 {
        543
    }
    else {
        345
    };
    
    645
}

fn in_while_condition() -> u64 {
    while return 42 {
        break;
    };
    
    745
}

fn in_enum() -> u64 {
    let _ = Enum::A((return 42, return 43));
    
    845
}

fn in_enum_multivariant() -> u64 {
    let _ = EnumMultivariant::B((return 42, return 43));
    
    945
}

fn helper_fun(x : u64, y : u64) -> u64 {
    x + y
}

fn in_fun_arg() -> u64 {
    let _ = helper_fun(return 42, return 43);

    1045
}

fn in_lazy_and() -> u64 {
    let _ = (return 42) && return 43;

    1145
}

fn in_lazy_or() -> u64 {
    let _ = (return 42) || return 43;

    1245
}

fn in_match_scrutinee() -> u64 {
    let _ = match return 42 {
        _ => 5411,
    };

    1345
}


fn main() -> u64 {
    assert(42 == in_init());
    assert(42 == in_array());
    assert(42 == in_length_1_array());
    assert(42 == in_length_2_array_first());
    assert(42 == in_length_2_array_second());
    assert(42 == in_tuple());
    assert(42 == in_struct());
    assert(42 == in_parentheses());
    assert(42 == in_arithmetic());
    assert(42 == in_if_condition());
    assert(42 == in_while_condition());
    assert(42 == in_enum());
    assert(42 == in_enum_multivariant());
    assert(42 == in_fun_arg());
    assert(42 == in_lazy_and());
    assert(42 == in_lazy_or());
    assert(42 == in_match_scrutinee());
    
    8193
}
