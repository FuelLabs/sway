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

// check: fn fn_implicit_ret_struct_0() -> { u64 }
// check: ret { u64 } $VAL

// check: fn fn_explicit_ret_struct_1() -> { u64 }
// check: ret { u64 } $VAL
