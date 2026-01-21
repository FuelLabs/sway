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

// ::check-ir::

// check: local { u64, u64 } __struct_init_0
// check: local { u64, u64 } record

// check: $(ptr_struct_init=$VAL) = get_local __ptr { u64, u64 }, __struct_init_0
// check: $(c_1=$VAL) = const u64 40
// check: $(c_2=$VAL) = const u64 2
// check: $(init_aggr=$VAL) = init_aggr $ptr_struct_init [$c_1, $c_2]
// check: $(init_aggr_val=$VAL) = load $init_aggr
// check: $(ptr_record=$VAL) = get_local __ptr { u64, u64 }, record
// check: store $init_aggr_val to $ptr_record
// check: $(ptr_record=$VAL) = get_local __ptr { u64, u64 }, record
// check: $(c_1=$VAL) = const u64 0
// check: $(ptr_record_a=$VAL) = get_elem_ptr $ptr_record, __ptr u64, $c_1
// check: $(load_a=$VAL) = load $ptr_record_a
// check: ret u64 $load_a

// ::check-ir-optimized::
// pass: lower-init-aggr

// check: $(ptr_struct_init=$VAL) = get_local __ptr { u64, u64 }, __struct_init_0
// check: $(c_1=$VAL) = const u64 0
// check: $(ptr_struct_init_a=$VAL) = get_elem_ptr $ptr_struct_init, __ptr u64, $c_1
// check: $(c_40=$VAL) = const u64 40
// check: store $c_40 to $ptr_struct_init_a
// check: $(c_1=$VAL) = const u64 1
// check: $(ptr_struct_init_b=$VAL) = get_elem_ptr $ptr_struct_init, __ptr u64, $c_1
// check: $(c_2=$VAL) = const u64 2
// check: store $c_2 to $ptr_struct_init_b
// check: $(load_struct_init=$VAL) = load $ptr_struct_init
// check: $(ptr_record=$VAL) = get_local __ptr { u64, u64 }, record
// check: store $load_struct_init to $ptr_record
// check: $(ptr_record=$VAL) = get_local __ptr { u64, u64 }, record
// check: $(c_1=$VAL) = const u64 0
// check: $(ptr_record_a=$VAL) = get_elem_ptr $ptr_record, __ptr u64, $c_1
// check: $(load_a=$VAL) = load $ptr_record_a
// check: ret u64 $load_a
