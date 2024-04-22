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
pub enum Enum_multivariant {
    A: (),
    B: (u64, u64),
}

// Illegal return types. Every function should report an error for incorrect return
// type, and a warning for unreachable code.

fn in_init() -> u64 {
    let _ = return;

    45
}

fn in_array() -> u64 {
    let _ = [return, return];
    
    145
}

// Arrays of length 1 are treated differently
fn in_length_1_array() -> u64 {
    let _ = [return];
    
    145
}

// The first element of an array is treated differently
fn in_length_2_array_first() -> u64 {
    let _ = [return, 0];
    
    145
}

// The first element of an array is treated differently
fn in_length_2_array_second() -> u64 {
    let _ = [0, return];
    
    145
}

fn in_tuple() -> u64 {
    let _ = (return, return);

    245
}

fn in_struct()  -> u64 {
    let _ = S { x: return, y: return };

    345 // TODO: Missing unreachable warning
}

fn in_parentheses()  -> u64 {
    let _ = (return);

    445
}

fn in_arithmetic_parse()  -> u64 {
    let _ = return + return;

    545
}

fn in_arithmetic_typecheck()  -> u64 {
    let _ = (return) + return;

    645 // TODO: Missing unreachable warning
}

fn in_if_condition() -> u64 {
    let _ = if return {
        457 // TODO: Missing unreachable warning
    }
    else {
        457 // TODO: Missing unreachable warning
    };

    745
}

fn in_while_condition() -> u64 {
    while return {
        break;
    };

    845
}

fn in_match_scrutinee() -> u64 {
    let _ = match return {
        _ => 458,
    }

    945
}
 
fn in_enum() -> u64 {
    let _ = Enum::A((return, return));
    
    1045 // TODO: Missing unreachable warning
}

fn in_enum_multivariant() -> u64 {
    let _ = Enum_multivariant::B((return, return));
    
    1145 // TODO: Missing unreachable warning
}

fn helper_fun(x : u64, y : u64) -> u64 {
    x + y
}

fn in_fun_arg() -> u64 {
    let _ = helper_fun(return, return);

    1245 // TODO: Missing unreachable warning
}

fn in_lazy_and_parse() -> u64 {
    let _ = return && return;

    1345
}

fn in_lazy_and_typecheck() -> u64 {
    let _ = (return) && return;

    1445
}

fn in_lazy_or_parse() -> u64 {
    let _ = return || return;

    1545
}

fn in_lazy_or_typecheck() -> u64 {
    let _ = (return) || return;

    1645
}


fn main() {
    assert(42 != in_init());
    assert(42 != in_array());
    assert(42 != in_length_1_array());
    assert(42 != in_length_2_array_first());
    assert(42 != in_length_2_array_second());
    assert(42 != in_tuple());
    assert(42 != in_struct());
    assert(42 != in_parentheses());
    assert(42 != in_arithmetic_parse());
    assert(42 != in_arithmetic_typecheck());
    assert(42 != in_if_condition());
    assert(42 != in_while_condition());
    assert(42 != in_match_scrutinee());
    assert(42 != in_enum());
    assert(42 != in_enum_multivariant());
    assert(42 != in_fun_arg());
    assert(42 != in_lazy_and_parse());
    assert(42 != in_lazy_and_typecheck());
    assert(42 != in_lazy_or_parse());
    assert(42 != in_lazy_or_typecheck());
}
