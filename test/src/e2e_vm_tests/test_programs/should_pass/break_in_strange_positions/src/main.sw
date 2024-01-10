script;

// This file tests the reported errors and warnings in various instances when a 'break'
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

// Legal uses. These should warn of unreachable code, but otherwise no error.

fn in_init() -> u64 {
    let mut i = 41;
    while i < 52 {
        i = i + 1;
        let _ = break;
        i = i + 1;
    }

    i
}

// Arrays of length 1 are treated differently from arrays of other lengths
fn in_length_1_array() -> u64 {
    let mut i = 41;
    while i < 52 {
        i = i + 1;
        let _ = [break];
        i = i + 1;
    }

    i
}

// The first element of an array is treated differently
fn in_length_2_array_first() -> u64 {
    let mut i = 41;
    while i < 52 {
        i = i + 1;
        let x = [break, 1];
        i = i + x[1];         // Missing warning
    }

    i
}

// The first element of an array is treated differently
fn in_length_2_array_second() -> u64 {
    let mut i = 41;
    while i < 52 {
        i = i + 1;
        let x = [1, break];
        i = i + x[0];         // Missing warning
    }
    
    i
}

fn in_array() -> u64 {
    let mut i = 41;
    while i < 52 {
        i = i + 1;
        let _ = [break, { i = i + 1; break }];
        i = i + 1;
    }
    
    i
}

fn in_tuple() -> u64 {
    let mut i = 41;
    while i < 52 {
        i = i + 1;
        let _ = (break, { i = i + 1; break });
        i = i + 1;
    }
    
    i
}

fn in_struct() -> u64 {
    let mut i = 41;
    while i < 52 {
        i = i + 1;
        let _ = S { x: break, y: { i = i + 1; break } };
        i = i + 1;
    }
    
    i
}

fn in_parentheses() -> u64 {
    let mut i = 41;
    while i < 52 {
        i = i + 1;
        let _ = (break);
        i = i + 1;
    }
    
    i
}

fn in_arithmetic() -> u64 {
    let mut i = 41;
    while i < 52 {
        i = i + 1;
        let _ = 1 + break;  // Missing warning
        i = i + 1;
    }
    
    i
}

fn in_if_condition() -> u64 {
    let mut i = 41;
    while i < 52 {
        i = i + 1;
        let _ = if break {
            543
        }
        else {
            345
        };
        i = i + 1;
    }
    
    i
}

fn in_while_condition() -> u64 {
    let mut i = 41;
    while i < 52 {
        i = i + 1;
        while break {
            i = i + 1;
        }
        i = i + 1;  // Missing warning
    }
    
    i
}

fn in_enum() -> u64 {
    let mut i = 41;
    while i < 52 {
        i = i + 1;
        let _ = Enum::A((break, { i = i + 1; break}));
        i = i + 1;
    }
    
    i
}

fn in_enum_multivariant() -> u64 {
    let mut i = 41;
    while i < 52 {
        i = i + 1;
        let _ = Enum_multivariant::B((break, { i = i + 1; break}));
        i = i + 1;
    }
    
    i
}

fn helper_fun(x : u64, y : u64) -> u64 {
    x + y
}

fn in_fun_arg() -> u64 {
    let mut i = 41;
    while i < 52 {
        i = i + 1;
        let _ = helper_fun(break, { i = i + 1; break});
        i = i + 1;
    }
    
    i
}

fn in_lazy_and() -> u64 {
    let mut i = 41;
    while i < 52 {
        i = i + 1;
        let _ = (break) && { i = i + 1; break};
        i = i + 1;
    }
    
    i
}

fn in_lazy_or() -> u64 {
    let mut i = 41;
    while i < 52 {
        i = i + 1;
        let _ = (break) || { i = i + 1; break};
        i = i + 1;
    }
    
    i
}

fn in_return() -> u64 {
    let mut i = 41;
    while i < 52 {
        i = i + 1;
        let _ = return break;
        i = i + 1;
    }
    
    i
}


fn main() -> u64 {
    assert(42 == in_init());
    assert(42 == in_length_1_array());
    assert(42 == in_length_2_array_first());
    assert(42 == in_length_2_array_second());
    assert(42 == in_array());
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
    assert(42 == in_return());

    8193
}
