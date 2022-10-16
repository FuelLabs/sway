script;

fn do_it(x: u64) -> u64 {
    x
}

fn generic<T>(input: T) -> T {
    do_it(input)
}

fn main() -> u64 {
    generic(7u64)
}
