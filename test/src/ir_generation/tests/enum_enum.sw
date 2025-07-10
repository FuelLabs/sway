script;

enum ABC {
    A: (),
    B: XYZ,
    C: (),
}

enum XYZ {
    X: (),
    Y: bool,
    Z: (),
}

fn main() {
    ABC::B(XYZ::X);
}

// ::check-ir::

// check: local { u64, ( () | { u64, ( () | bool | () ) } | () ) } $ID
// check: local { u64, ( () | bool | () ) } $ID

// check: $(abc_ptr=$VAL) = get_local __ptr { u64, ( () | { u64, ( () | bool | () ) } | () ) }, $ID
// check: $(idx_val=$VAL) = const u64 0
// check: $(abc_tag_ptr=$VAL) = get_elem_ptr $abc_ptr, __ptr u64, $idx_val
// check: $(b_tag=$VAL) = const u64 1
// check: store $b_tag to $abc_tag_ptr

// check: $(xyz_ptr=$VAL) = get_local __ptr { u64, ( () | bool | () ) }, $ID
// check: $(idx_val=$VAL) = const u64 0
// check: $(xyz_tag_ptr=$VAL) = get_elem_ptr $xyz_ptr, __ptr u64, $idx_val
// check: $(x_tag=$VAL) = const u64 0
// check: store $x_tag to $xyz_tag_ptr

// check: $(xyz_val=$VAL) = load $xyz_ptr

// check: $(idx_val=$VAL) = const u64 1
// check: $(abc_val_ptr=$VAL) = get_elem_ptr $abc_ptr, __ptr { u64, ( () | bool | () ) }, $idx_val
// check: store $xyz_val to $abc_val_ptr

// check: load $abc_ptr
