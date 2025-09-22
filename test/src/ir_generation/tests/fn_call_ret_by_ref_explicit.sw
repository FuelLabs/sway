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

// There are return value '__ret_val' temporaries and there are other '__aggr_memcpy_' temporaries and in
// this test we want to match the return values specifically.  If we ever rename them from
// '__ret_val_' to something else then this test will fail.

// regex: RET_VAL_ID=__ret_val\d*

// check: fn main() -> u64
// check: local { u64, u64, u64 } $(ret_for_call_0=$RET_VAL_ID)
// check: local { u64, u64, u64 } $(ret_for_call_1=$RET_VAL_ID)

// check: $(ret_arg_0=$VAL) = get_local __ptr { u64, u64, u64 }, $ret_for_call_0
// check: $(ret_val_0=$VAL) = call $(a_func=$ID)($ID, $ID, $ID, $ret_arg_0)
// check: $(tmp_ptr=$VAL) = get_local __ptr { u64, u64, u64 }, $ID
// check: mem_copy_val $tmp_ptr, $ret_arg_0

// check: $(idx_val=$VAL) = const u64 1
// check: $(field_val=$VAL) = get_elem_ptr $tmp_ptr, __ptr u64, $idx_val
// check: load $field_val

// check: $(ret_arg_1=$VAL) = get_local __ptr { u64, u64, u64 }, $ret_for_call_1
// check: $(ret_val_1=$VAL) = call $a_func($ID, $ID, $ID, $ret_arg_1)
// check: $(tmp_ptr=$VAL) = get_local __ptr { u64, u64, u64 }, $ID
// check: mem_copy_val $tmp_ptr, $ret_arg_1

// check: $(idx_val=$VAL) = const u64 2
// check: $(field_val=$VAL) = get_elem_ptr $tmp_ptr, __ptr u64, $idx_val
// check: load $field_val

// fn a()...
//
// check: fn $a_func($ID $MD: bool, $ID $MD: u64, $ID $MD: u64, $(ret_val_arg=$ID): __ptr { u64, u64, u64 }) -> ()
// check: cbr $ID, $(block_0=$ID)(), $(block_1=$ID)()

// check: $block_0():
// check: mem_copy_val $ret_val_arg, $VAL
// check: ret ()

// check: $block_1():
// check: mem_copy_val $ret_val_arg, $VAL
// check: ret ()
