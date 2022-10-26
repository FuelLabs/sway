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

// check: fn $ID(__ret_value $MD: mut ptr { u64 }) -> { u64 }
// check:mem_copy __ret_value, $VAL, 8

// check: fn $ID(__ret_value $MD: mut ptr { u64 }) -> { u64 }
// check:mem_copy __ret_value, $VAL, 8
