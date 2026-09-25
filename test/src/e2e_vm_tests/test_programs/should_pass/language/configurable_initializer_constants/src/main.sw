script;

mod constants;

const BASE: u64 = 1;
const NEXT: u64 = BASE + 1;

configurable {
    FIRST: u64 = NEXT,
    SECOND: u64 = constants::FIRST + 1,
    THIRD: u64 = if true { NEXT } else { FIRST },
}

fn main() -> u64 { FIRST + SECOND + THIRD }
