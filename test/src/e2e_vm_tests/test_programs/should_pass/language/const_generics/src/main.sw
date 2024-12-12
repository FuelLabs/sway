script;

trait A {
    fn len() -> u64;
}

impl<T, const N: u64> A for [T; N] {
    fn len(self) -> u64 {
        N
    }
}

fn main() -> u64 {
    1
}