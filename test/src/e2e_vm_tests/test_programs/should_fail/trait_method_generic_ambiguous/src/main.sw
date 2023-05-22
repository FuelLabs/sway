script;

trait Trait<T> {
    fn method(self) -> u64;
}

struct S1 {
    s1: u64
}

struct S2 {
    s2: u64
}

impl Trait<S1> for u64 {
    fn method(self) -> u64 {
        1
    }
}

impl Trait<S2> for u64 {
    fn method(self) -> u64 {
        2
    }
}

fn main() {
    let _v1 = 42.method();
}