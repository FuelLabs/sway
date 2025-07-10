script;

pub struct MyStruct {
    value: u64,
}

fn fn_implicit_ret_struct() -> MyStruct {
    let s = MyStruct {
        value: 0,
    };
    s
}

fn fn_explicit_ret_struct() -> MyStruct {
    let s = MyStruct {
        value: 0,
    };
    return s;
}

fn main() {
    fn_implicit_ret_struct();
    fn_explicit_ret_struct();
}

// check: fn $ID() -> { u64 }
// check: $VAL = get_local __ptr { u64 }, s
// check: $(ptr_val=$VAL) = get_local __ptr { u64 }, s
// check: $(ret_val=$VAL) = load $ptr_val
// check: ret { u64 } $ret_val

// check: fn $ID() -> { u64 }
// check: $VAL = get_local __ptr { u64 }, s
// check: $(ptr_val=$VAL) = get_local __ptr { u64 }, s
// check: $(ret_val=$VAL) = load $ptr_val
// check: ret { u64 } $ret_val
