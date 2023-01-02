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

// check: local { u64, u64 } record

// check: $(record_undef=$VAL) = get_local { u64, u64 } $ID
// check: $(forty=$VAL) = const u64 40
// check: $(record_0=$VAL) = insert_value $record_undef, { u64, u64 }, $forty, 0
// check: $(two=$VAL) = const u64 2
// check: $(record=$VAL) = insert_value $record_0, { u64, u64 }, $two, 1
// check: $(record_var=$VAL) = get_local { u64, u64 } record
// check: store $record to $record_var

// check: $(record_var=$VAL) = get_local { u64, u64 } record
// check: $(record_field=$VAL) = extract_value $record_var, { u64, u64 }, 0
// check: ret u64 $record_field
