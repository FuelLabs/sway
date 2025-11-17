script;

enum ABC {
    A: (),
    B: XYZ,
    C: (),
}

struct XYZ {
    x: b256,
    y: bool,
    z: u64,
}

fn main() {
    ABC::B(XYZ {
        x: 0x0001010101010101000101010101010100010101010101010001010101010101,
        y: true,
        z: 53,
    });
}

// ::check-ir::

// check: $(temp_ptr_0=$VAL) = get_local __ptr { u64, ( () | { b256, bool, u64 } | () ) }, $(=__anon_\d+)
// check: $(idx_0=$VAL) = const u64 0
// check: $(tag_ptr=$VAL) = get_elem_ptr $temp_ptr_0, __ptr u64, $idx_0
// check: $(tag_1=$VAL) = const u64 1
// check: store $tag_1 to $tag_ptr

// check: $(temp_ptr_1=$VAL) = get_local __ptr { b256, bool, u64 }, $(=__struct_init_\d+)
// check: $(x_val=$VAL) = const b256 0x0001010101010101000101010101010100010101010101010001010101010101
// check: $(t=$VAL) = const bool true
// check: $(fif3=$VAL) = const u64 53
// check: $(init_aggr=$VAL) = init_aggr $temp_ptr_1 [$x_val, $t, $fif3]
// check: $(load_init_aggr=$VAL) = load $init_aggr
// check: $(idx_1=$VAL) = const u64 1
// check: $(idx_2=$VAL) = const u64 1
// check: $(variant_b_ptr=$VAL) = get_elem_ptr $temp_ptr_0, __ptr { b256, bool, u64 }, $idx_1, $idx_2
// check: store $load_init_aggr to $variant_b_ptr

// ::check-ir-optimized::
// pass: lower-init-aggr

// check: $(temp_ptr_0=$VAL) = get_local __ptr { u64, ( () | { b256, bool, u64 } | () ) }, $(=__anon_\d+)
// check: $(idx_0=$VAL) = const u64 0
// check: $(tag_ptr=$VAL) = get_elem_ptr $temp_ptr_0, __ptr u64, $idx_0
// check: $(tag_1=$VAL) = const u64 1
// check: store $tag_1 to $tag_ptr

// check: $(temp_ptr_1=$VAL) = get_local __ptr { b256, bool, u64 }, $(=__struct_init_\d+)
// check: $(idx_0=$VAL) = const u64 0
// check: $(x_ptr=$VAL) = get_elem_ptr $temp_ptr_1, __ptr b256, $idx_0
// check: $(x_val=$VAL) = const b256 0x0001010101010101000101010101010100010101010101010001010101010101
// check: store $x_val to $x_ptr

// check: $(idx_1=$VAL) = const u64 1
// check: $(y_ptr=$VAL) = get_elem_ptr $temp_ptr_1, __ptr bool, $idx_1
// check: $(t=$VAL) = const bool true
// check: store $t to $y_ptr

// check: $(idx_2=$VAL) = const u64 2
// check: $(z_ptr=$VAL) = get_elem_ptr $temp_ptr_1, __ptr u64, $idx_2
// check: $(fif3=$VAL) = const u64 53
// check: store $fif3 to $z_ptr

// check: $(xyz_val=$VAL) = load $temp_ptr_1

// check: $(idx_1=$VAL) = const u64 1
// check: $(idx_2=$VAL) = const u64 1
// check: $(variant_val_ptr=$VAL) = get_elem_ptr $temp_ptr_0, __ptr { b256, bool, u64 }, $idx_1, $idx_2
// check: store $xyz_val to $variant_val_ptr

// check: load $temp_ptr_0
