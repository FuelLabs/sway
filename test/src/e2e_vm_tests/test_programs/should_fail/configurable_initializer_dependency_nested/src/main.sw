script;

configurable {
    FIRST: u64 = 0,
    SECOND: u64 = nested(),
}

fn nested() -> u64 {
    let value = { __add(FIRST, 1) };
    value
}

fn main() -> u64 { SECOND }
