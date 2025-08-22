script;

fn main() -> u64 {
    let record = Record {
        a: 40,
        b: 2,
    };
    record.a
}

struct Record {
    a: u64,
    b: u64,
}

// check: local { u64, u64 } record

// check: $(temp_var=$VAL) = get_local __ptr { u64, u64 }, __anon_0

// check: $(idx_val=$VAL) = const u64 0
// check: $(temp_ptr=$VAL) = get_elem_ptr $temp_var, __ptr u64, $idx_val
// check: $(forty=$VAL) = const u64 40
// check: store $forty to $temp_ptr
// check: $(idx_val=$VAL) = const u64 1
// check: $(temp_ptr=$VAL) = get_elem_ptr $temp_var, __ptr u64, $idx_val
// check: $(two=$VAL) = const u64 2
// check: store $two to $temp_ptr

// check: $(temp_val=$VAL) = load $temp_var
// check: $(record_ptr=$VAL) = get_local __ptr { u64, u64 }, record
// check: store $temp_val to $record_ptr

// check: $(record_ptr=$VAL) = get_local __ptr { u64, u64 }, record
// check: $(idx_val=$VAL) = const u64 0
// check: $(field_ptr=$VAL) = get_elem_ptr $record_ptr, __ptr u64, $idx_val
// check: $(field_val=$VAL) = load $field_ptr
// check: ret u64 $field_val
