script;

struct S<T> {
}

impl<T> S<T> {
    fn foo(self) -> u64 {
        __size_of::<T>()
    }
}

fn main() -> u64 {
    let s = S {};
    s.foo()
}
