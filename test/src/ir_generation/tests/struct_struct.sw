script;

fn main() -> u64 {
    let record = Record {
        a: 0x0102030405060708010203040506070801020304050607080102030405060708,
        b: Entry {
            c: true,
            d: 76,
        }
    };
    record.b.d
}

struct Record {
    a: b256,
    b: Entry,
}

struct Entry {
    c: bool,
    d: u64,
}


// check: local { b256, { bool, u64 } } record

// check: $(tmp_ptr=$VAL) = get_local __ptr { bool, u64 }, $ID

// check: $(idx_val=$VAL) = const u64 0
// check: $(c_ptr=$VAL) = get_elem_ptr $tmp_ptr, __ptr bool, $idx_val
// check: $(t=$VAL) = const bool true
// check: store $t to $c_ptr

// check: $(idx_val=$VAL) = const u64 1
// check: $(d_ptr=$VAL) = get_elem_ptr $tmp_ptr, __ptr u64, $idx_val
// check: $(sevsix=$VAL) = const u64 76
// check: store $sevsix to $d_ptr

// check: $(b_val=$VAL) = load $tmp_ptr

// check: $(tmp_ptr=$VAL) = get_local __ptr { b256, { bool, u64 } }, $ID

// check: $(idx_val=$VAL) = const u64 0
// check: $(a_ptr=$VAL) = get_elem_ptr $tmp_ptr, __ptr b256, $idx_val
// check: $(addr=$VAL) = const b256 0x0102030405060708010203040506070801020304050607080102030405060708
// check: store $addr to $a_ptr

// check: $(idx_val=$VAL) = const u64 1
// check: $(b_ptr=$VAL) = get_elem_ptr $tmp_ptr, __ptr { bool, u64 }, $idx_val
// check: store $b_val to $b_ptr

// check: $(record_val=$VAL) = load $tmp_ptr

// check: $(record_ptr=$VAL) = get_local __ptr { b256, { bool, u64 } }, record
// check: store $record_val to $record_ptr

// check: $(record_ptr=$VAL) = get_local __ptr { b256, { bool, u64 } }, record
// check: $(idx_val=$VAL) = const u64 1
// check: $(b_ptr=$VAL) = get_elem_ptr $record_ptr, __ptr { bool, u64 }, $idx_val

// check: $(idx_val=$VAL) = const u64 1
// check: $(d_ptr=$VAL) = get_elem_ptr $b_ptr, __ptr u64, $idx_val
// check: $(d_val=$VAL) = load $d_ptr

// check: ret u64 $d_val
