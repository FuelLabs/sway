// target-fuelvm

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

// There are return value '__ret_val' temporaries and there are other '__anon_' temporaries and in
// this test we want to match the return values specifically.  If we ever rename them from
// '__ret_val_' to something else then this test will fail.

// regex: RET_VAL_ID=__ret_val\d*

// check: fn main() -> u64
// check: local { u64, u64, u64 } $(ret_for_call_0=$RET_VAL_ID)
// check: local { u64, u64, u64 } $(ret_for_call_1=$RET_VAL_ID)

// check: $(ret_arg_0=$VAL) = get_local ptr { u64, u64, u64 }, $ret_for_call_0
// check: $(ret_val_0=$VAL) = call $(a_func=$ID)($ID, $ID, $ID, $ret_arg_0)
// check: $(tmp_ptr_0=$VAL) = ptr_to_int $ret_val_0 to u64
// check: $(ret_0=$VAL) = int_to_ptr $tmp_ptr_0 to ptr { u64, u64, u64 }

// check: $(idx_val=$VAL) = const u64 1
// check: $(field_val=$VAL) = get_elem_ptr $ret_0, ptr u64, $idx_val
// check: load $field_val

// check: $(ret_arg_1=$VAL) = get_local ptr { u64, u64, u64 }, $ret_for_call_1
// check: $(ret_val_1=$VAL) = call $a_func($ID, $ID, $ID, $ret_arg_1)
// check: $(tmp_ptr_1=$VAL) = ptr_to_int $ret_val_1 to u64
// check: $(ret_1=$VAL) = int_to_ptr $tmp_ptr_1 to ptr { u64, u64, u64 }

// check: $(idx_val=$VAL) = const u64 2
// check: $(field_val=$VAL) = get_elem_ptr $ret_1, ptr u64, $idx_val
// check: load $field_val

// fn a()...
//
// check: fn $a_func($ID $MD: bool, $ID $MD: u64, $ID $MD: u64, $(ret_val_arg=$ID): ptr { u64, u64, u64 }) -> ptr { u64, u64, u64 }
// check: cbr $ID, $(block_0=$ID)(), $(block_1=$ID)()

// check: $block_0():
// check: mem_copy_val $ret_val_arg, $VAL
// check: ret ptr { u64, u64, u64 } $ret_val_arg

// check: $block_1():
// check: mem_copy_val $ret_val_arg, $VAL
// check: ret ptr { u64, u64, u64 } $ret_val_arg
