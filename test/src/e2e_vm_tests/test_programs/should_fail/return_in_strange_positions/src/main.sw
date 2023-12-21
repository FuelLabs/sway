script;

use std::assert::assert;

// This file tests the reported errors and warnings in various instances when a 'return'
// occurs in a non-statement position. This is allowed, but will often result in
// unreachable code or similar warning situations.

pub struct S { x : u64, y : u64, }

// Legal return types. These should warn of unreachable code, but otherwise no error.

fn in_init() -> u64 {
    let _ = return 42;
    
    45
}

fn in_array() -> u64 {
    let _ = [return 42, return 43];
   
    145
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
    let _ = return 42 + return 43;

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

// Legal return type. Matching on the type is unimplemented.

fn in_match_scrutinee() -> u64 {
    match return 42 {
        _ => 845,
    }
}

// Incorrect return types. These should fail because of the 'return'.

fn in_init_non_value_return() {
    let _ = return;
}

fn in_array_non_value_return() {
    let _ = [return, return];
}

fn in_tuple_non_value_return() {
    let _ = (return, return);
}

fn in_struct_non_value_return() {
    let _ = S { x: return, y: return };
}

fn in_parentheses_non_value_return() {
    let _ = (return);
}

fn in_arithmetic_non_value_return() {
    let _ = return + return;
}

fn in_if_condition_non_value_return() {
    let _ = if return {
        543
    }
    else {
        345
    };
}

fn in_while_condition_non_value_return() {
    while return {
        break;
    };
}

fn in_match_scrutinee_non_value_return() {
    match return {
        _ => 845,
    }
}
 

// Copy paste for all expressions, there are not that many :-)
// Also wild things like:
// return 43 + return 42;

// Then copy paste the same for functions returning unit.
//fn in_array_unit() {
//   let _ = [return, return];
//   
//   assert(false);
//}

// ...

fn main() {
   assert(42 == in_init());
   assert(42 == in_array());
   assert(42 == in_tuple());
   assert(42 == in_struct());
   assert(42 == in_parentheses());
   assert(42 == in_arithmetic());
   assert(42 == in_if_condition());
   assert(42 == in_while_condition());
   assert(42 == in_match_scrutinee());
   // ...
   
   in_init_non_value_return();
   in_array_non_value_return();
   in_tuple_non_value_return();
   in_struct_non_value_return();
   in_parentheses_non_value_return();
   in_arithmetic_non_value_return();
   in_if_condition_non_value_return();
   in_while_condition_non_value_return();
   in_match_scrutinee_non_value_return();
   // ...  
}
