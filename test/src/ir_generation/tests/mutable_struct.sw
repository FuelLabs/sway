script;

fn main() -> u64 {
    let mut record = Record {
        a: 40,
        b: 2,
    };
    record.a = 50;
    record.b
}

struct Record {
    a: u64,
    b: u64,
}

// check: local mut { u64, u64 } record

// The first get_local when initialising record:
// check: get_local __ptr { u64, u64 }, record

// The second one when mutating.
// check: $(rec_var=$VAL) = get_local __ptr { u64, u64 }, record
// check: $(idx_0=$VAL) = const u64 0
// check: $(a_ptr=$VAL) = get_elem_ptr $rec_var, __ptr u64, $idx_0
// check: $(fifty=$VAL) = const u64 50
// check: store $fifty to $a_ptr
