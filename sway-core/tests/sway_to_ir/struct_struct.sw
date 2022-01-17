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
