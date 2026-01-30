script;

fn main() -> u64 {
    let record = Record {
        a: 40,
        b: [0u64, 0u64, 50u64, 0u64],
        c: 0,
        d: 0,
    };
    record.b[0]
}

struct Record {
    a: u64,
    b: [u64; 4],
    c: u64,
    d: u64,
}

// ::check-ir::
// check: local { u64, [u64; 4], u64, u64 } __struct_init_0

// ::check-ir-optimized::
// pass: lower-init-aggr

// check: local [u64; 4] __array_init_0
// check: local { u64, [u64; 4], u64, u64 } __struct_init_0
// check: $(v1v1=$VAL) = get_local __ptr { u64, [u64; 4], u64, u64 }, __struct_init_0
// check: $(v2v1=$VAL) = get_local __ptr [u64; 4], __array_init_0, !5
// check: mem_clear_val v1v1
// check-not: store
// check: $(v5v1=$VAL) = const u64 50
// check: store $v5v1 to
// check-not: store
// check: $(v9v1=$VAL) = const u64 40
// check: store $v9v1 to
// check-not: store
// check: $VAL = load $v1v1