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

// check: local { u64, u64 } record

// The first get_local when initialising record:
// check: get_local { u64, u64 } record

// The second one when mutating.
// check: $(rec_var=$VAL) = get_local { u64, u64 } record
// check: $(fifty=$VAL) = const u64 50
// check: insert_value $rec_var, { u64, u64 }, $fifty, 0
