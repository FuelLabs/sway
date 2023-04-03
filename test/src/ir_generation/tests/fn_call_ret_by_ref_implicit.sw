// target-fuelvm

script;

fn a(x: u64) -> (u64, u64, u64) {
    (x, x, x)
}

fn main() -> u64 {
    a(a(11).1).2
}

// ::check-ir::

// check: entry fn main() -> u64
// check: local { u64, u64, u64 } $(=__ret_val.*)
// check: local { u64, u64, u64 } $(=__ret_val.*)

// check: $(ret_val_ptr=$VAL) = get_local ptr { u64, u64, u64 }, $(=__ret_val.*)
// check: $(arg_11=$VAL) = const u64 11
// check: $(call_a_ret_ptr=$VAL) = call $(a_fn=$ID)($arg_11, $ret_val_ptr)
// check: $(temp_0=$VAL) = get_local ptr { u64, u64, u64 }, $(=__anon_\d+)
// check: mem_copy_val $temp_0, $call_a_ret_ptr

// check: $(idx_1=$VAL) = const u64 1
// check: $(field_1_ptr=$VAL) = get_elem_ptr $temp_0, ptr u64, $idx_1
// check: $(field_1_val=$VAL) = load $field_1_ptr
// check: $(ret_val_ptr=$VAL) = get_local ptr { u64, u64, u64 }, $(=__ret_val.*)
// check: $(call_a_ret_ptr=$VAL) = call $a_fn($field_1_val, $ret_val_ptr)
// check: $(temp_1=$VAL) = get_local ptr { u64, u64, u64 }, $(=__anon_\d+)
// check: mem_copy_val $temp_1, $call_a_ret_ptr

// check: $(idx_2=$VAL) = const u64 2
// check: $(field_2_ptr=$VAL) = get_elem_ptr $temp_1, ptr u64, $idx_2
// check: $(field_2_val=$VAL) = load $field_2_ptr
// check: ret u64 $field_2_val

// check: fn $a_fn($(x_arg=$ID) $MD: u64, $(ret_val_arg_ptr=$ID): ptr { u64, u64, u64 }) -> ptr { u64, u64, u64 }

// check: $(temp_ptr=$VAL) = get_local ptr { u64, u64, u64 }, $(=__anon_\d+)

// check: $(idx_0=$VAL) = const u64 0
// check: $(field_0_ptr=$VAL) = get_elem_ptr $temp_ptr, ptr u64, $idx_0
// check: store $x_arg to $field_0_ptr

// check: $(idx_1=$VAL) = const u64 1
// check: $(field_1_ptr=$VAL) = get_elem_ptr $temp_ptr, ptr u64, $idx_1
// check: store $x_arg to $field_1_ptr

// check: $(idx_2=$VAL) = const u64 2
// check: $(field_2_ptr=$VAL) = get_elem_ptr $temp_ptr, ptr u64, $idx_2
// check: store $x_arg to $field_2_ptr

// check: mem_copy_val $ret_val_arg_ptr, $temp_ptr
// check: ret ptr { u64, u64, u64 } $ret_val_arg_ptr
