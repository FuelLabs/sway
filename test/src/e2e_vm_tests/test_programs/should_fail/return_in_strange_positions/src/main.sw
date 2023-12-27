script;

// This file tests the reported errors and warnings in various instances when a 'return'
// occurs in a non-statement position. This is allowed, but will often result in
// unreachable code or similar warning situations.

pub struct S { x : u64, y : u64, }
pub enum Enum {
    A: (u64, u64),
}

// Legal return type. Matching on the type is unimplemented.

fn in_match_scrutinee_legal_return() -> u64 {
    match return 42 {
        _ => 5411,
    }

    1145
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

fn in_tuple() -> u64 {
    let _ = (return, return);

    245
}

fn in_struct()  -> u64 {
    let _ = S { x: return, y: return };

    345
}

fn in_parentheses()  -> u64 {
    let _ = (return);

    445
}

fn in_arithmetic()  -> u64 {
    let _ = return + return;

    545
}

fn in_if_condition() -> u64 {
    let _ = if return {
        456
    }
    else {
        457
    };

    645
}

fn in_while_condition() -> u64 {
    while return {
        break;
    };

    745
}

fn in_match_scrutinee() -> u64 {
    let _ = match return {
        _ => 458,
    }

    845
}
 
fn in_enum() -> u64 {
    let _ = Enum::A((return, return));
    
    945
}

fn helper_fun(x : u64, y : u64) -> u64 {
    x + y
}

fn in_fun_arg() -> u64 {
    let _ = helper_fun(return, return);

    1045
}


fn main() {
   assert(42 == in_match_scrutinee_legal_return());

   assert(42 != in_init());
   assert(42 != in_array());
   assert(42 != in_tuple());
   assert(42 != in_struct());
   assert(42 != in_parentheses());
   assert(42 != in_arithmetic());
   assert(42 != in_if_condition());
   assert(42 != in_while_condition());
   assert(42 != in_match_scrutinee());
   assert(42 != in_enum());
   assert(42 != in_fun_arg());
}
