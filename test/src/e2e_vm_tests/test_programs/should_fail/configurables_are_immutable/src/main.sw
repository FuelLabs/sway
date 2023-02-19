script;

configurable {
    C1: u64 = 5,
}

fn main() -> u64 {
    C1 = 6;
    C1
}
