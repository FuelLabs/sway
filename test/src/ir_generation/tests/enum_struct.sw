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

// check: $(enum_undef=$VAL) = const { u64, ( () | { b256, bool, u64 } | () ) } { u64 undef, ( () | { b256, bool, u64 } | () ) undef }
// check: $(enum_tag=$VAL) = const u64 1
// check: $(enum_tagged=$VAL) = insert_value $enum_undef, { u64, ( () | { b256, bool, u64 } | () ) }, $enum_tag, 0
// check: $(struct_undef=$VAL) = const { b256, bool, u64 } { b256 undef, bool undef, u64 undef }
// check: $(struct_0=$VAL) = insert_value $struct_undef, { b256, bool, u64 }, $VAL, 0
// check: $(struct_01=$VAL) = insert_value $struct_0, { b256, bool, u64 }, $VAL, 1,
// check: $(struct_012=$VAL) = insert_value $struct_01, { b256, bool, u64 }, $VAL, 2
// check: insert_value $enum_tagged, { u64, ( () | { b256, bool, u64 } | () ) }, $struct_012, 1
