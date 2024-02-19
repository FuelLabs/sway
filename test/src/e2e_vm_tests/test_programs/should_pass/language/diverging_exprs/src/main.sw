script;

fn diverge_in_let_body() -> u64 {
    let _x: bool =  {
        return 5;
    };
    123
}

struct Foo {
    x: bool,
    y: u32,
}

fn diverge_in_struct_0() -> u64 {
    let _foo: Foo = Foo {
        x:  {
            return 5;
            true
        },
        y: 123,
    };
    123
}

fn diverge_in_struct_1() -> u64 {
    let _foo: Foo = Foo {
        x: true,
        y:  {
            return 5;
            123
        },
    };
    123
}

fn diverge_in_tuple_0() -> u64 {
    let _tuple: (bool, u32) = (
         {
            return 5;
        },
        123,
    );
    123
}

fn diverge_in_tuple_1() -> u64 {
    let _tuple: (bool, u32) = (
        true,
         {
            return 5;
        },
    );
    123
}

fn diverge_in_array() -> u64 {
    let _arr: [bool; 2] = [ {
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
    let _b: bool = if  { return 5;     } { true } else { false };
    123
}

fn diverge_in_if_then() -> u64 {
    let _b: bool = if true { return 5; } else { false };
    123
}

fn diverge_in_if_else() -> u64 {
    let _b: bool = if false { true } else { return 5; };
    123
}

fn diverge_in_match_condition() -> u64 {
    match  {
        return 5;
        true
    } {
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

fn diverge_in_match_branch_2() -> u64 {
    let _m:! = match false {
        true => {
            return 5;
        },
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

fn diverge_in_array_index_index() -> u64 {
    let arr: [bool; 2] = [true, false];
    let _b: bool = arr[ {
        return 5;
    }];
    123
}

fn diverge_in_op_not() -> u64 {
    let _b: bool = ! {
        return 5;
    };
    123
}

fn diverge_in_op_add_rhs() -> u64 {
    let _x: u32 = 1u32 + ( {
        return 5;
        1u32
    });
    123
}

fn diverge_in_logical_and_lhs() -> u64 {
    let _b: bool = ( {
        return 5;
        true
    }) && true;
    123
}

fn diverge_in_logical_and_rhs() -> u64 {
    let _b: bool = true && ( {
        return 5;
        true
    });
    123
}

fn diverge_in_logical_or_lhs() -> u64 {
    let _b: bool = ( {
        return 5;
        true
    }) || true;
    123
}

fn diverge_in_logical_or_rhs() -> u64 {
    let _b: bool = false || ( {
        return 5;
        true
    });
    123
}

fn diverge_in_reassignment() -> u64 {
    let mut _b: bool = true;
    _b =  {
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

fn diverge_in_eq() -> u64 {
    let mut _b: bool = true;
    _b = {
        return 5;
    } == {
        return 6;
    };
    123
}

fn diverge_in_lt() -> u64 {
    let mut _b: bool = true;
    _b = {
        return 5;
    } < {
        return 6;
    };
    123
}

fn diverge_in_gt() -> u64 {
    let mut _b: bool = true;
    _b = {
        return 5;
    } > {
        return 6;
    };
    123
}

#[inline(never)]
fn diverge_in_if_with_std_revert(cond: bool) -> (u64, u64) {
    let result1 = if cond == true {
        revert(0)
    } else {
        5
    };

    let result2 = if cond == false {
        5
    } else {
        revert(0)
    };

    (result1, result2)
}

#[inline(never)]
fn diverge_in_if_with_revert_intrinsic(cond: bool) -> (u64, u64) {
    let result1 = if cond == true {
        __revert(0)
    } else {
        5
    };

    let result2 = if cond == false {
        5
    } else {
        __revert(0)
    };

    (result1, result2)
}

#[inline(never)]
fn diverge_in_match_with_std_revert(cond: bool) -> (u64, u64) {
    let result1 = match cond {
        true => revert(0),
        false => 5,
    };

    let result2 = match cond {
        false => 5,
        true => revert(0),
    };

    (result1, result2)
}

#[inline(never)]
fn diverge_in_match_with_revert_intrinsic(cond: bool) -> (u64, u64) {
    let result1 = match cond {
        true => __revert(0),
        false => 5,
    };

    let result2 = match cond {
        false => 5,
        true => __revert(0),
    };

    (result1, result2)
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
    assert(5 == diverge_in_match_branch_2());
    assert(5 == diverge_in_while_condition());
    assert(5 == diverge_in_while_body());
    assert(5 == diverge_in_func_arg());
    assert(5 == diverge_in_array_index_index());
    assert(5 == diverge_with_if_else(true));
    assert(1 == diverge_with_if_else(false));
    assert(5 == diverge_in_op_not());
    assert(5 == diverge_in_op_add_rhs());
    assert(5 == diverge_in_logical_and_lhs());
    assert(5 == diverge_in_logical_and_rhs());
    assert(5 == diverge_in_logical_or_lhs());
    assert(5 == diverge_in_logical_or_rhs());
    assert(5 == diverge_in_reassignment());
    assert(5 == diverge_in_eq());
    assert(5 == diverge_in_lt());
    assert(5 == diverge_in_gt());

    let result = diverge_in_if_with_std_revert(false);
    assert(result.0 == 5);
    assert(result.1 == 5);

    let result = diverge_in_if_with_revert_intrinsic(false);
    assert(result.0 == 5);
    assert(result.1 == 5);

    let result = diverge_in_match_with_std_revert(false);
    assert(result.0 == 5);
    assert(result.1 == 5);

    let result = diverge_in_match_with_revert_intrinsic(false);
    assert(result.0 == 5);
    assert(result.1 == 5);

    // Test type coercion
    if false {
        let _: u8 = __revert(1);      // Ok.  Never -> u8.
        let _: u8 = { return 123 };  // Ok.  Never -> u8.
        let _: ! = __revert(1);       // Ok.  Never -> Never.
        let _: ! = { return 123 };   // Ok.  Never -> Never.
    }

    42
}
