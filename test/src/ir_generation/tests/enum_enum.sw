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

// check: $(outer_undef=$VAL) = const { u64, ( () | { u64, ( () | bool | () ) } | () ) } { u64 undef, ( () | { u64, ( () | bool | () ) } | () ) undef }
// check: $(outer_tag=$VAL) = const u64 1
// check: $(outer_tagged=$VAL) = insert_value $outer_undef, { u64, ( () | { u64, ( () | bool | () ) } | () ) }, $outer_tag, 0
// check: $(inner_undef=$VAL) = const { u64, ( () | bool | () ) } { u64 undef, ( () | bool | () ) undef }
// check: $(inner_tag=$VAL) = const u64 0
// check: $(inner_tagged=$VAL) = insert_value $inner_undef, { u64, ( () | bool | () ) }, $inner_tag, 0
// check: insert_value $outer_tagged, { u64, ( () | { u64, ( () | bool | () ) } | () ) }, $inner_tagged, 1
