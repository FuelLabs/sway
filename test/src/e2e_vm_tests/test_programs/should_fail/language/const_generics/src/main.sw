script;

trait A {
    fn f(self) -> u64;
}

impl<T, const N: u64> A for [T; N] {
    #[allow(dead_code)]
    fn f(self) -> u64 {
        1
    }
}

fn main() {
}
