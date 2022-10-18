script;

fn a(p: bool, x: u64, y: u64) -> (u64, u64, u64) {
    if p {
        return (x, x, x);
    }

    return (y, y, y);
}

fn main() -> u64 {
    a(false, a(true, 11, 22).1, 33).2
}

// ::check-ir::

// check: fn main() -> u64
// check: local mut ptr { u64, u64, u64 } $(ret_for_call_0=$ID)
// check: local mut ptr { u64, u64, u64 } $(ret_for_call_1=$ID)

// check: $(ret_arg_0=$ID) = get_ptr mut ptr { u64, u64, u64 } $ret_for_call_0, ptr { u64, u64, u64 }, 0
// check: $(ret_val_0=$ID) = call $(a_func=$ID)($ID, $ID, $ID, $ret_arg_0)
// check: extract_value $ret_val_0, { u64, u64, u64 }, 1

// check: $(ret_arg_1=$ID) = get_ptr mut ptr { u64, u64, u64 } $ret_for_call_1, ptr { u64, u64, u64 }, 0
// check: $(ret_val_1=$ID) = call $a_func($ID, $ID, $ID, $ret_arg_1)
// check extract_value $ret_val_1, { u64, u64, u64 }, 2

// fn a()...
//
// check: fn $a_func($ID $MD: bool, $ID $MD: u64, $ID $MD: u64, __ret_value $MD: mut ptr { u64, u64, u64 }) -> { u64, u64, u64 }
// check: cbr $ID, $(block_0=$ID)(), $(block_1=$ID)()

// A single mem_copy for each explicit return:
//
// check: $block_0():
// check: mem_copy __ret_value
// not: mem_copy __ret_value
// check: ret { u64, u64, u64 } $ID

// check: $block_1():
// check: mem_copy __ret_value
// not: mem_copy __ret_value
// check: ret { u64, u64, u64 } $ID
