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

// ::check-ir::

// check: local { bool, { u64, ( () | () | u64 ) } } __struct_init_0
// check: local { bool, { u64, ( () | () | u64 ) } } record

// check: $(ptr_struct_init=$VAL) = get_local __ptr { bool, { u64, ( () | () | u64 ) } }, __struct_init_0
// check: mem_clear_val $ptr_struct_init
// check: $(load_struct_init=$VAL) = load $ptr_struct_init
// check: $(ptr_record=$VAL) = get_local __ptr { bool, { u64, ( () | () | u64 ) } }, record
// check: store $load_struct_init to $ptr_record
// check: $(ptr_record=$VAL) = get_local __ptr { bool, { u64, ( () | () | u64 ) } }, record
// check: $(id_0=$VAL) = const u64 0
// check: $(ptr_record_a=$VAL) = get_elem_ptr $ptr_record, __ptr bool, $id_0
// check: $(record_a_val=$VAL) = load $ptr_record_a
// check: ret bool $record_a_val

// ::check-ir-optimized::
// pass: lower-init-aggr

// check: local { bool, { u64, ( () | () | u64 ) } } __struct_init_0
// check: local { bool, { u64, ( () | () | u64 ) } } record

// check: $(ptr_struct_init=$VAL) = get_local __ptr { bool, { u64, ( () | () | u64 ) } }, __struct_init_0
// check: mem_clear_val $ptr_struct_init
// check: $(load_struct_init=$VAL) = load $ptr_struct_init
// check: $(ptr_record=$VAL) = get_local __ptr { bool, { u64, ( () | () | u64 ) } }, record
// check: store $load_struct_init to $ptr_record
// check: $(ptr_record=$VAL) = get_local __ptr { bool, { u64, ( () | () | u64 ) } }, record
// check: $(id_0=$VAL) = const u64 0
// check: $(ptr_record_a=$VAL) = get_elem_ptr $ptr_record, __ptr bool, $id_0
// check: $(record_a_val=$VAL) = load $ptr_record_a
// check: ret bool $record_a_val