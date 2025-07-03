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


// check: local { bool, { u64, ( () | () | u64 ) } } record

// check: $(enum_ptr=$VAL) = get_local __ptr { u64, ( () | () | u64 ) }, $ID
// check: $(idx_val=$VAL) = const u64 0
// check: $(tag_ptr=$VAL) = get_elem_ptr $enum_ptr, __ptr u64, $idx_val
// check: $(zero=$VAL) = const u64 0
// check: store $zero to $tag_ptr
// check: $(enum_val=$VAL) = load $enum_ptr

// check: $(temp_struct_ptr=$VAL) = get_local __ptr { bool, { u64, ( () | () | u64 ) } }, $ID
// check: $(idx_val=$VAL) = const u64 0
// check: $(field_ptr=$VAL) = get_elem_ptr $temp_struct_ptr, __ptr bool, $idx_val
// check: $(f=$VAL) = const bool false
// check: store $f to $field_ptr

// check: $(idx_val=$VAL) = const u64 1
// check: $(field_ptr=$VAL) = get_elem_ptr $temp_struct_ptr, __ptr { u64, ( () | () | u64 ) }, $idx_val
// check: store $enum_val to $field_ptr
// check: $(temp_struct_val=$VAL) = load $temp_struct_ptr

// check: $(record_ptr=$VAL) = get_local __ptr { bool, { u64, ( () | () | u64 ) } }, record
// check: store $temp_struct_val to $record_ptr

// check: $(record_ptr=$VAL) = get_local __ptr { bool, { u64, ( () | () | u64 ) } }, record
// check: $(idx_val=$VAL) = const u64 0
// check: $(field_ptr=$VAL) = get_elem_ptr $record_ptr, __ptr bool, $idx_val
// check: $(field_val=$VAL) = load $field_ptr

// check: ret bool $field_val
