//! Initialization of aggregates whose fields (or whole values) come from
//! branching expressions (`if`/`else`, `else if` chains) and from function
//! calls. This tests `init_aggr`s whose initializers are computed in
//! predecessor blocks and merged, as well as nested `init_aggr`s inside `if`s.
library;

use ::types::*;

#[inline(never)]
fn get_no_nesting_all_zeros() -> NoNesting {
    NoNesting {
        a: 0,
        b: false,
        c: 0u256,
        d: b256::zero(),
        u: (),
    }
}

#[test]
fn test_from_if_true() {
    from_if(true);
}

#[test]
fn test_from_if_false() {
    from_if(false);
}

#[inline(never)]
pub fn from_if(value: bool) {
    let no_nesting = if value {
        NoNesting {
            a: 111,
            b: false,
            c: 0u256,
            d: b256::zero(),
            u: (),
        }
    } else {
        NoNesting {
            a: 222,
            b: true,
            c: 42u256,
            d: b256::zero(),
            u: (),
        }
    };

    assert_no_nesting(
        no_nesting,
        if value {
            111
        } else {
            222
        },
        if value {
            false
        } else {
            true
        },
        if value {
            0u256
        } else {
            42u256
        },
        b256::zero(),
    );

    let no_nesting = if value {
        NoNesting {
            a: 333,
            b: false,
            c: 0u256,
            d: b256::zero(),
            u: (),
        }
    } else {
        get_no_nesting_all_zeros()
    };

    if value {
        assert_no_nesting(no_nesting, 333, false, 0u256, b256::zero());
    } else {
        assert_no_nesting_all_zeros(no_nesting);
    }

    let s = Nested {
        n1: if value {
            NoNesting {
                a: 999,
                b: false,
                c: 444u256,
                d: b256::zero(),
                u: (),
            }
        } else {
            NoNesting {
                a: 999,
                b: true,
                c: 555u256,
                d: b256::zero(),
                u: (),
            }
        },
        n2: NoNesting {
            a: 999,
            b: false,
            c: 666u256,
            d: b256::zero(),
            u: (),
        },
    };

    if value {
        assert_no_nesting(s.n1, 999, false, 444u256, b256::zero());
    } else {
        assert_no_nesting(s.n1, 999, true, 555u256, b256::zero());
    }
    assert_no_nesting(s.n2, 999, false, 666u256, b256::zero());
}

#[test]
fn test_from_if_isolated_true() {
    from_if_isolated(true);
}

#[test]
fn test_from_if_isolated_false() {
    from_if_isolated(false);
}

#[inline(never)]
pub fn from_if_isolated(value: bool) {
    let s = Nested {
        n1: if value {
            NoNesting {
                a: 999,
                b: false,
                c: 444u256,
                d: b256::zero(),
                u: (),
            }
        } else {
            NoNesting {
                a: 999,
                b: true,
                c: 555u256,
                d: b256::zero(),
                u: (),
            }
        },
        n2: NoNesting::default(),
    };

    if value {
        assert_no_nesting(s.n1, 999, false, 444u256, b256::zero());
    } else {
        assert_no_nesting(s.n1, 999, true, 555u256, b256::zero());
    }
    assert_no_nesting_all_zeros(s.n2);
}

#[test]
fn test_from_if_struct_true() {
    from_if_struct(true);
}

#[test]
fn test_from_if_struct_false() {
    from_if_struct(false);
}

#[inline(never)]
pub fn from_if_struct(value: bool) {
    let s = Struct {
        x: 1111,
        simple: if value {
            Simple {
                a: 111,
                b: 22222,
                c: true,
                d: 33333u256,
            }
        } else {
            create_some_simple(42)
        },
        b: true,
    };

    if value {
        assert_simple(s.simple, 111, 22222, true, 33333u256);
    } else {
        assert_simple(s.simple, 42, 2, true, 3u256);
    }
}

#[test]
fn test_from_if_struct_only_inits_true() {
    from_if_struct_only_inits(true);
}

#[test]
fn test_from_if_struct_only_inits_false() {
    from_if_struct_only_inits(false);
}

#[inline(never)]
pub fn from_if_struct_only_inits(value: bool) {
    let s = Struct {
        x: 1111,
        simple: if value {
            Simple {
                a: 111,
                b: 22222,
                c: true,
                d: 33333u256,
            }
        } else {
            Simple {
                a: 222,
                b: 33333,
                c: true,
                d: 44444u256,
            }
        },
        b: true,
    };

    if value {
        assert_simple(s.simple, 111, 22222, true, 33333u256);
    } else {
        assert_simple(s.simple, 222, 33333, true, 44444u256);
    }
}

#[test]
fn test_from_multiple_if_else_struct_value_1() {
    from_multiple_if_else_struct(1);
}

#[test]
fn test_from_multiple_if_else_struct_value_2() {
    from_multiple_if_else_struct(2);
}

#[test]
fn test_from_multiple_if_else_struct_value_42() {
    from_multiple_if_else_struct(42);
}

#[inline(never)]
pub fn from_multiple_if_else_struct(value: u64) {
    let s = Struct {
        x: 1111,
        simple: if value == 1 {
            Simple {
                a: 111,
                b: 22222,
                c: true,
                d: 33333u256,
            }
        } else if value == 2 {
            create_some_simple(1)
        } else {
            create_some_simple(42)
        },
        b: true,
    };

    if value == 1 {
        assert_simple(s.simple, 111, 22222, true, 33333u256);
    } else if value == 2 {
        assert_simple(s.simple, 1, 2, true, 3u256);
    } else {
        assert_simple(s.simple, 42, 2, true, 3u256);
    }
}

#[test]
fn test_from_multiple_if_else_struct_only_inits_value_1() {
    from_multiple_if_else_struct_only_inits(1);
}

#[test]
fn test_from_multiple_if_else_struct_only_inits_value_2() {
    from_multiple_if_else_struct_only_inits(2);
}

#[test]
fn test_from_multiple_if_else_struct_only_inits_value_42() {
    from_multiple_if_else_struct_only_inits(42);
}

#[inline(never)]
pub fn from_multiple_if_else_struct_only_inits(value: u64) {
    let s = Struct {
        x: 1111,
        simple: if value == 1 {
            Simple {
                a: 111,
                b: 22222,
                c: true,
                d: 33333u256,
            }
        } else if value == 2 {
            Simple {
                a: 222,
                b: 33333,
                c: true,
                d: 44444u256,
            }
        } else {
            Simple {
                a: 255,
                b: 44444,
                c: true,
                d: 55555u256,
            }
        },
        b: true,
    };

    if value == 1 {
        assert_simple(s.simple, 111, 22222, true, 33333u256);
    } else if value == 2 {
        assert_simple(s.simple, 222, 33333, true, 44444u256);
    } else {
        assert_simple(s.simple, 255, 44444, true, 55555u256);
    }
}
