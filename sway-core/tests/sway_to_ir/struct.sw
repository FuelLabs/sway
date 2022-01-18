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
