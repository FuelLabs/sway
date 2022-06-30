script;

fn bla(x: u64) -> u64 {
    x + 1
}

fn main() -> u64 {
    const X = bla(0);
    X
}
