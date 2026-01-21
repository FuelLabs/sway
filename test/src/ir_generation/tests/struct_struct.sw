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

// ::check-ir::

// check: local { b256, { bool, u64 } } __struct_init_0
// check: local { bool, u64 } __struct_init_1
// check: local { b256, { bool, u64 } } record

// check: $(record_ptr=$VAL) = get_local __ptr { b256, { bool, u64 } }, __struct_init_0
// check: $(entry_ptr=$VAL) = get_local __ptr { bool, u64 }, __struct_init_1
// check: $(t=$VAL) = const bool true
// check: $(sevsix=$VAL) = const u64 76
// check: $(init_aggr_entry=$VAL) = init_aggr $entry_ptr [$t, $sevsix]
// check: $(load_entry=$VAL) = load $init_aggr_entry
// check: $(b256_val=$VAL) = const b256 0x0102030405060708010203040506070801020304050607080102030405060708
// check: $(init_aggr_record=$VAL) = init_aggr $record_ptr [$b256_val, $load_entry]
// check: $(load_record=$VAL) = load $init_aggr_record
// check: $(record_ptr=$VAL) = get_local __ptr { b256, { bool, u64 } }, record
// check: store $load_record to $record_ptr
// check: $(record_ptr=$VAL) = get_local __ptr { b256, { bool, u64 } }, record
// check: $(idx_val=$VAL) = const u64 1
// check: $(b_ptr=$VAL) = get_elem_ptr $record_ptr, __ptr { bool, u64 }, $idx_val
// check: $(idx_val=$VAL) = const u64 1
// check: $(d_ptr=$VAL) = get_elem_ptr $b_ptr, __ptr u64, $idx_val
// check: $(d_val=$VAL) = load $d_ptr
// check: ret u64 $d_val

// ::check-ir-optimized::
// pass: lower-init-aggr

// check: $(struct_init=$VAL) = get_local __ptr { b256, { bool, u64 } }, __struct_init_0
// check: $(c_1=$VAL) = const u64 1
// check: $(ptr_b=$VAL) = get_elem_ptr $struct_init, __ptr { bool, u64 }, $c_1
// check: $(c_1=$VAL) = const u64 1
// check: $(c_2=$VAL) = const u64 0
// check: $(ptr_b_c=$VAL) = get_elem_ptr $struct_init, __ptr bool, $c_1, $c_2
// check: $(c_true=$VAL) = const bool true
// check: store $c_true to $ptr_b_c
// check: $(c_1=$VAL) = const u64 1
// check: $(c_2=$VAL) = const u64 1
// check: $(ptr_b_d=$VAL) = get_elem_ptr $struct_init, __ptr u64, $c_1, $c_2
// check: $(const_76=$VAL) = const u64 76
// check: store $const_76 to $ptr_b_d
// check: $(load_b=$VAL) = load $ptr_b
// check: $(c_0=$VAL) = const u64 0
// check: $(ptr_a=$VAL) = get_elem_ptr $struct_init, __ptr b256, $c_0
// check: $(const_a=$VAL) = const b256 0x0102030405060708010203040506070801020304050607080102030405060708
// check: store $const_a to $ptr_a
// check: $(load_struct=$VAL) = load $struct_init
// check: $(ptr_record=$VAL) = get_local __ptr { b256, { bool, u64 } }, record
// check: store $load_struct to $ptr_record
// check: $(ptr_record=$VAL) = get_local __ptr { b256, { bool, u64 } }, record
// check: $(c_1=$VAL) = const u64 1
// check: $(ptr_b=$VAL) = get_elem_ptr $ptr_record, __ptr { bool, u64 }, $c_1
// check: $(c_2=$VAL) = const u64 1
// check: $(ptr_d=$VAL) = get_elem_ptr $ptr_b, __ptr u64, $c_2
// check: $(load_d=$VAL) = load $ptr_d
// check: ret u64 $load_d
