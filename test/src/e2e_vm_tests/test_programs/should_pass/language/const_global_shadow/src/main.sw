script;

const GLOBAL_VAL: u64 = 1;

fn main() -> u64 {
    const GLOBAL_VAL = 100;
    const LOCAL_VAL = GLOBAL_VAL;
    LOCAL_VAL
}