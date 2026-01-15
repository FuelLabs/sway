// This test proves that https://github.com/FuelLabs/sway/issues/7520 is fixed.
library;

const GLOBAL_CONST: u64 = 1122;

struct Struct {
    x: u64,
}

// Using `#[test]` is a trick to be able to use library for avoiding `std` dependency
// and still have the IR compilation phase run.
#[test]
fn test_local_vars() {
    let outer = 0u64;
    let mut mut_outer = 100u64;
    let mut s = Struct { x: 0 };

    const LOCAL_CONST: u64 = 2211;

    const OK_USES_GLOBAL_CONST: u64 = GLOBAL_CONST;

    const OK_USES_LOCAL_CONST: u64 = LOCAL_CONST;

    const ERR_USES_OUTER: u64 = outer;

    const ERR_USES_MUT_OUTER: u64 = mut_outer;

    const ERR_USES_MUT_STRUCT_FIELD: u64 = s.x;

    const ERR_USES_OUTER_IN_BLOCK: u64 = {
        outer
    };

    const OK_USES_LOCAL_OUTER_IN_BLOCK: u64 = {
        let outer = 10u64;
        outer
    };

    const ERR_USES_OUTER_IN_BLOCK_WITH_LOCAL_OUTER: u64 = {
        {
            let outer = 10u64;
            let _ = outer;
        }
        outer
    };

    const ERR_USES_MUT_OUTER_IN_REASSIGNMENT: u64 = {
        mut_outer = 200u64;
        0u64
    };

    const ERR_USES_OUTER_IN_REASSIGNMENT_RHS: u64 = {
        let mut array = [1u64, 2u64, 3u64];
        array[outer]
    };

    const ERR_USES_OUTER_PROJECTION_ACCESS_IN_REASSIGNMENT_RHS: u64 = {
        s.x = 1;
        0u64
    };

    const ERR_USES_OUTER_IN_NESTED_CONSTS: u64 = {
        const NESTED_CONST: u64 = outer;
        NESTED_CONST
    };

    const ERR_USES_NESTED_OUTER_IN_NESTED_CONSTS: u64 = {
        let outer = 55u64;
        const NESTED_CONST: u64 = outer;
        NESTED_CONST
    };

    const OK_USES_NESTED_CONST_IN_NESTED_CONSTS: u64 = {
        const NESTED_CONST: u64 = 100u64;
        {
            const OTHER_NESTED_CONST: u64 = NESTED_CONST;
            OTHER_NESTED_CONST
        }
    };

    const OK_MATCH_DESUGARING_LOCAL_VARS: u64 = match 100u64 {
        x => x,
    };

    fn_args_1(outer, mut_outer);
    fn_args_2(outer, mut_outer);
    fn_args_3(outer, mut_outer);
    fn_args_4(outer, mut_outer);
    fn_args_5(outer, mut_outer);
    fn_args_6(outer, mut_outer);
    fn_args_7(outer, mut_outer);
    fn_args_8(outer, mut_outer);
}

fn fn_args_1(outer: u64, ref mut _mut_outer: u64) {
    const ERR_USES_OUTER: u64 = outer;
}

fn fn_args_2(_outer: u64, ref mut mut_outer: u64) {
    const ERR_USES_MUT_OUTER: u64 = mut_outer;
}

fn fn_args_3(outer: u64, ref mut _mut_outer: u64) {
    const ERR_USES_OUTER_IN_BLOCK: u64 = {
        outer
    };
}

fn fn_args_4(outer: u64, ref mut _mut_outer: u64) {
    const ERR_USES_OUTER_IN_BLOCK_WITH_LOCAL_OUTER: u64 = {
        {
            let outer = 10u64;
            let _ = outer;
        }
        outer
    };
}

fn fn_args_5(_outer: u64, ref mut mut_outer: u64) {
    const ERR_USES_MUT_OUTER_IN_REASSIGNMENT: u64 = {
        mut_outer = 200u64;
        0u64
    };
}

fn fn_args_6(outer: u64, ref mut _mut_outer: u64) {
    const ERR_USES_OUTER_IN_REASSIGNMENT_RHS: u64 = {
        let mut array = [1u64, 2u64, 3u64];
        array[outer]
    };
}

fn fn_args_7(outer: u64, ref mut _mut_outer: u64) {
    const ERR_USES_OUTER_IN_NESTED_CONSTS: u64 = {
        const NESTED_CONST: u64 = outer;
        NESTED_CONST
    };
}

fn fn_args_8(_outer: u64, ref mut _mut_outer: u64) {
    const ERR_USES_NESTED_OUTER_IN_NESTED_CONSTS: u64 = {
        let outer = 55u64;
        const NESTED_CONST: u64 = outer;
        NESTED_CONST
    };
}
