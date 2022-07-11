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

// check: local mut ptr { u64, u64 } record

// The first get_ptr when initialising record:
// check: get_ptr mut ptr { u64, u64 } record, ptr { u64, u64 }, 0

// The second one when mutating.
// check: $(rec_ptr=$VAL) = get_ptr mut ptr { u64, u64 } record, ptr { u64, u64 }, 0
// check: $(fifty=$VAL) = const u64 50
// check: insert_value $rec_ptr, { u64, u64 }, $fifty, 0
