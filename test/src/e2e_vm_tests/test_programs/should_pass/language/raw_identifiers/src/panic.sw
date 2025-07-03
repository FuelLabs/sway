library;

pub fn r#panic(x: u64) -> u64 {
    x
}

pub fn call_panic() -> u64 {
    r#panic(42)
}