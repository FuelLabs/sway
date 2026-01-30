script;

fn main() -> u64 {
    let record = Record {
        a: 40,
        b: 0,
        c: 0,
        d: 0,
    };
    record.a
}

struct Record {
    a: u64,
    b: u64,
    c: u64,
    d: u64,
}

// ::check-ir::
// check: local { u64, u64, u64, u64 } __struct_init_0

// ::check-ir-optimized::
// pass: lower-init-aggr

// check: local { u64, u64, u64, u64 } __struct_init_0
// check: $(v1v1=$VAL) = get_local __ptr { u64, u64, u64, u64 }, __struct_init_0,
// check: mem_clear_val $v1v1
// check: $(v16v1=$VAL) = const u64 0
// check: $(v17v1=$VAL) = get_elem_ptr $v1v1, __ptr u64, $v16v1
// check-not: const u64 0
// check-not: $v16v1
// check: $(v2v1=$VAL) = const u64 40, !5
// check: store $v2v1 to $v17v1, !4