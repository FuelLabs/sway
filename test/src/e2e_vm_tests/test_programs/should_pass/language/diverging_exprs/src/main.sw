script;

use std::assert::assert;

fn revert<T>() -> T {
    let code = 1u64;
    __revert(code)
}

fn diverge_in_let_body() -> u64 {
    let x: bool =  {
        return 5;
    };
    123
}

struct Foo {
    x: bool,
    y: u32,
}

fn diverge_in_struct_0() -> u64 {
    let foo: Foo = Foo {
        x:  {
            return 5;
            true
        },
        y: 123,
    };
    123
}

fn diverge_in_struct_1() -> u64 {
    let foo: Foo = Foo {
        x: true,
        y:  {
            return 5;
            123
        },
    };
    123
}

fn diverge_in_tuple_0() -> u64 {
    let tuple: (bool, u32) = (
         {
            return 5;
        },
        123,
    );
    123
}

fn diverge_in_tuple_1() -> u64 {
    let tuple: (bool, u32) = (
        true,
         {
            return 5;
        },
    );
    123
}

fn diverge_in_array() -> u64 {
    let arr: [bool; 2] = [ {
            return 5;
        }; 2    ];
    123
}

fn diverge_in_return() -> u64 {
    return  {
        return 5;
        6
    };
}

fn diverge_in_if_condition() -> u64 {
    let b: bool = if  { return 5;     } { true } else { false };
    123
}

fn diverge_in_if_then() -> u64 {
    let b: bool = if true { return 5; } else { false };
    123
}

fn diverge_in_if_else() -> u64 {
    let b: bool = if false { true } else { return 5; };
    123
}

fn diverge_in_match_condition() -> u64 {
    match  {
        return 5;
        true
    } {
        true => 23,
        false => 56,
    }
}

fn diverge_in_match_branch_0() -> u64 {
    match true {
        true => {
            return 5;
        },
        false => (),
    };
    123
}

fn diverge_in_match_branch_1() -> u64 {
    match false {
        true => (),
        false => {
            return 5;
        },
    };
    123
}

fn diverge_in_while_condition() -> u64 {
    while  {
        return 5;
    } {    }
    123
}

fn diverge_in_while_body() -> u64 {
    while true {
        return 5;
    }
    123
}

fn func(arg: bool) -> u64 {
    if arg { 123 } else { 456 }
}

fn diverge_in_func_arg() -> u64 {
    func( {
        return 5;
    })
}

fn diverge_in_array_index_array() -> u64 {
    let b: bool =  {
        return 5;
        [true, false]
    }[0];
    123
}

fn diverge_in_array_index_index() -> u64 {
    let arr: [bool; 2] = [true, false];
    let b: bool = arr[ {
        return 5;
    }];
    123
}


// Disabled due to https://github.com/FuelLabs/sway/issues/3061
/*fn diverge_in_op_not() -> u64 {
    let b: bool = ! {
        return 5;
    };
    123
}*/

fn diverge_in_op_add_lhs() -> u64 {
    let x: u32 = ( {
        return 5;
        1u32
    }) + 2u32;
    123
}

fn diverge_in_op_add_rhs() -> u64 {
    let x: u32 = 1u32 + ( {
        return 5;
        1u32
    });
    123
}

fn diverge_in_logical_and_lhs() -> u64 {
    let b: bool = ( {
        return 5;
        true
    }) && true;
    123
}

fn diverge_in_logical_and_rhs() -> u64 {
    let b: bool = true && ( {
        return 5;
        true
    });
    123
}

fn diverge_in_logical_or_lhs() -> u64 {
    let b: bool = ( {
        return 5;
        true
    }) || true;
    123
}

fn diverge_in_logical_or_rhs() -> u64 {
    let b: bool = false || ( {
        return 5;
        true
    });
    123
}

fn diverge_in_reassignment() -> u64 {
    let mut b: bool = true;
    b =  {
        return 5;
    };
    123
}

fn diverge_with_if_else(b: bool) -> u64 {
    let x: u64 = if b {
        return 5;
    } else {
       return 1;
    };

    return x;
}

fn main() -> u64 {
    assert(5 == diverge_in_let_body());
    assert(5 == diverge_in_struct_0());
    assert(5 == diverge_in_struct_1());
    assert(5 == diverge_in_tuple_0());
    assert(5 == diverge_in_tuple_1());
    assert(5 == diverge_in_array());
    assert(5 == diverge_in_return());
    assert(5 == diverge_in_if_condition());
    assert(5 == diverge_in_if_then());
    assert(5 == diverge_in_if_else());
    assert(5 == diverge_in_match_condition());
    assert(5 == diverge_in_match_branch_0());
    assert(5 == diverge_in_match_branch_1());
    assert(5 == diverge_in_while_condition());
    assert(5 == diverge_in_while_body());
    assert(5 == diverge_in_func_arg());
    assert(5 == diverge_in_array_index_array());
    assert(5 == diverge_in_array_index_index());
    assert(5 == diverge_with_if_else(true));
    assert(1 == diverge_with_if_else(false));

    // Disabled due to https://github.com/FuelLabs/sway/issues/3061
    // assert(5 == diverge_in_op_not());

    assert(5 == diverge_in_op_add_lhs());
    assert(5 == diverge_in_op_add_rhs());
    assert(5 == diverge_in_logical_and_lhs());
    assert(5 == diverge_in_logical_and_rhs());
    assert(5 == diverge_in_logical_or_lhs());
    assert(5 == diverge_in_logical_or_rhs());
    assert(5 == diverge_in_reassignment());

    42
}
