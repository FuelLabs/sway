script;

const GLOBAL_VAL: u64 = 99;

fn main() -> u64 {
    const LOCAL_VAL = 1;
    GLOBAL_VAL + LOCAL_VAL
}
