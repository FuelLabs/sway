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
pub enum EnumMultivariant {
    A: (),
    B: (u64, u64),
}

// Legal uses. These should warn of unreachable code, but otherwise no error.

fn in_init() -> u64 {
    let mut i = 31;
    while i < 42 {
        i = i + 1;
        let _ = continue;
        i = 100;
    }

    i
}

// Arrays of length 1 are treated differently from arrays of other lengths
fn in_length_1_array() -> u64 {
    let mut i = 31;
    while i < 42 {
        i = i + 1;
        let _ = [continue];
        i = 100;
    }
    
    i
}
 
// The first element of an array is treated differently
fn in_length_2_array_first() -> u64 {
    let mut i = 31;
    while i < 42 {
        i = i + 1;
        let x = [continue, { i = 90; 100 } ];
        i = x[1];
    }
    
    i
}
 
// The first element of an array is treated differently
fn in_length_2_array_second() -> u64 {
    let mut i = 31;
    while i < 32 {
        i = i + 1;
        let x = [ { i = 42; 100 }, continue];
        i = x[0];
    }
    
    i
}
 
fn in_array() -> u64 {
    let mut i = 31;
    while i < 42 {
        i = i + 1;
        let _ = [continue, { i = 90; continue }];
        i = 100;
    }
    
    i
}
 
fn in_tuple() -> u64 {
    let mut i = 31;
    while i < 42 {
        i = i + 1;
        let _ = (continue, { i = 90; continue });
        i = 100;
    }
    
    i
}
 
fn in_struct() -> u64 {
    let mut i = 31;
    while i < 42 {
        i = i + 1;
        let _ = S { x: continue, y: { i = 90; continue } };
        i = 100;
    }
    
    i
}
 
fn in_parentheses() -> u64 {
    let mut i = 31;
    while i < 42 {
        i = i + 1;
        let _ = (continue);
        i = 100;
    }
    
    i
}

fn in_arithmetic() -> u64 {
    let mut i = 31;
    while i < 42 {
        i = i + 1;
        let _ = 1 + continue;
        i = 100;
    }
    
    i
}

fn in_if_condition() -> u64 {
    let mut i = 31;
    while i < 42 {
        i = i + 1;
        let _ = if continue {
            i = 90;
        }
        else {
            i = 100;
        };
    }
    
    i
}
 
fn in_while_condition() -> u64 {
    let mut i = 31;
    while i < 42 {
        i = i + 1;
        while continue {
            i = 90;
        }
        i = 100;
    }
    
    i
}

fn in_enum() -> u64 {
    let mut i = 31;
    while i < 42 {
        i = i + 1;
        let _ = Enum::A((continue, { i = 90; continue}));
        i = 100;
    }
    
    i
}
 
fn in_enum_multivariant() -> u64 {
    let mut i = 31;
    while i < 42 {
        i = i + 1;
        let _ = EnumMultivariant::B((continue, { i = 90; continue}));
        i = 100;
    }
    
    i
}

fn helper_fun(x : u64, y : u64) -> u64 {
    x + y
}

fn in_fun_arg() -> u64 {
    let mut i = 31;
    while i < 42 {
        i = i + 1;
        let _ = helper_fun(continue, { i = 90; continue});
        i = 100;
    }
    
    i
}

fn in_lazy_and() -> u64 {
    let mut i = 31;
    while i < 42 {
        i = i + 1;
        let _ = (continue) && { i = 90; continue};
        i = 100;
    }
    
    i
}

fn in_lazy_or() -> u64 {
    let mut i = 31;
    while i < 42 {
        i = i + 1;
        let _ = (continue) || { i = 90; continue};
        i = 100;
    }
    
    i
}

fn in_return() -> u64 {
    let mut i = 31;
    while i < 42 {
        i = i + 1;
        let _ = return continue;
        i = 100;
    }
    
    i
}

fn in_match_scrutinee_continue() -> u64 {
    let mut i = 32;
    while i < 42 {
        i = i + 1;
        match continue {
            _ => return 5411,
        }
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
    assert(42 == in_match_scrutinee_continue());

    8193
}
