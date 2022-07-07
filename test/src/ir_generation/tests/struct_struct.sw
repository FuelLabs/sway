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

// check: local ptr { b256, { bool, u64 } } record

// check: $(entry_undef=$VAL) = const { bool, u64 } { bool undef, u64 undef }
// check: $(t=$VAL) = const bool true
// check: $(entry_0=$VAL) = insert_value $entry_undef, { bool, u64 }, $t, 0
// check: $(sevsix=$VAL) = const u64 76
// check: $(entry=$VAL) = insert_value $entry_0, { bool, u64 }, $sevsix, 1
// check: $(record_undef=$VAL) = const { b256, { bool, u64 } } { b256 undef, { bool, u64 } { bool undef, u64 undef } }
// check: $(b256_lit=$VAL) = const b256 0x0102030405060708010203040506070801020304050607080102030405060708
// check: $(record_0=$VAL) = insert_value $record_undef, { b256, { bool, u64 } }, $b256_lit, 0
// check: $(record=$VAL) = insert_value $record_0, { b256, { bool, u64 } }, $entry, 1
// check: $(record_ptr=$VAL) = get_ptr ptr { b256, { bool, u64 } } record, ptr { b256, { bool, u64 } }, 0
// check: store $record, ptr $record_ptr
// check: $(record_ptr=$VAL) = get_ptr ptr { b256, { bool, u64 } } record, ptr { b256, { bool, u64 } }, 0
// check: $(inner=$VAL) = extract_value $record_ptr, { b256, { bool, u64 } }, 1
// check: $(inner_field=$VAL) = extract_value $inner, { bool, u64 }, 1
// check: ret u64 $inner_field
