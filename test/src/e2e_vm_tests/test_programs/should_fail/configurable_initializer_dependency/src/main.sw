script;

configurable {
    FIRST: u64 = 0,
    SECOND: u64 = FIRST + 1,
}

fn main() -> u64 { SECOND }
