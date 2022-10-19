script;

fn a(x: u64) -> (u64, u64, u64) {
    (x, x, x)
}

fn main() -> u64 {
    a(a(11).1).2
}

// ::check-ir::

// check: fn main() -> u64
// check: local mut ptr { u64, u64, u64 } $(ret_for_call_0=$ID)
// check: local mut ptr { u64, u64, u64 } $(ret_for_call_1=$ID)

// check: $(ret_arg_0=$ID) = get_ptr mut ptr { u64, u64, u64 } $ret_for_call_0, ptr { u64, u64, u64 }, 0
// check: $(ret_val_0=$ID) = call $(a_func=$ID)($ID, $ret_arg_0)
// check: extract_value $ret_val_0, { u64, u64, u64 }, 1

// check: $(ret_arg_1=$ID) = get_ptr mut ptr { u64, u64, u64 } $ret_for_call_1, ptr { u64, u64, u64 }, 0
// check: $(ret_val_1=$ID) = call $a_func($ID, $ret_arg_1)
// check extract_value $ret_val_1, { u64, u64, u64 }, 2

// check: fn $a_func($ID $MD: u64, __ret_value $MD: mut ptr { u64, u64, u64 }) -> { u64, u64, u64 }
// check: mem_copy __ret_value
// There should be only a single mem_copy
// not: mem_copy __ret_value
// check: ret { u64, u64, u64 }
