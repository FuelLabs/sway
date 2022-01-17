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
