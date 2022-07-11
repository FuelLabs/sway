script;

fn main() -> bool {
    let record = Record {
        a: false,
        b: Fruit::Apple,
    };
    record.a
}

struct Record {
    a: bool,
    b: Fruit,
}

enum Fruit {
    Apple: (),
    Banana: (),
    Grapes: u64,
}

// check: local ptr { bool, { u64, ( () | () | u64 ) } } record

// check: $(enum_undef=$VAL) = const { u64, ( () | () | u64 ) } { u64 undef, ( () | () | u64 ) undef }
// check: $(zero=$VAL) = const u64 0
// check: $(enum_tagged=$VAL) = insert_value $enum_undef, { u64, ( () | () | u64 ) }, $zero, 0
// check: $(struct_undef=$VAL) = const { bool, { u64, ( () | () | u64 ) } } { bool undef, { u64, ( () | () | u64 ) } { u64 undef, ( () | () | u64 ) undef } }
// check: $(f=$VAL) = const bool false
// check: $(struct_0=$VAL) = insert_value $struct_undef, { bool, { u64, ( () | () | u64 ) } }, $f, 0
// check: $(struct_init=$VAL) = insert_value $struct_0, { bool, { u64, ( () | () | u64 ) } }, $enum_tagged, 1
// check: $(record_ptr=$VAL) = get_ptr ptr { bool, { u64, ( () | () | u64 ) } } record, ptr { bool, { u64, ( () | () | u64 ) } }, 0
// check: store $struct_init, ptr $record_ptr
