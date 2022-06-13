script;

const GLOBAL_NUM = a_number(1, 2, 3);

fn a_number(_a: u64, _b: u64, _c: u64) -> u64 {
    42
}

fn main() -> u64 {
    let a = a_number(4, 5, 6);
    GLOBAL_NUM
}
