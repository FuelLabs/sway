script;

trait A {
    fn lenxxx(self) -> u64;
}

impl<T, const N: u64> A for [T; N] {
    fn lenxxx(self) -> u64 {
        N
    }
}

fn main(a: [u64; 1]) {
    let a = [9].lenxxx();
    let b = [9, 10].lenxxx();

    if (a + b) != 0 {
        revert(0)
    }
}