library;

trait Cat {
    fn speak(self) -> u64;
}

struct S<T> {
    x: T,
}

impl S<u32> {
    fn foo(self) -> u64 {
        1
    }
}

impl<Z> S<Z>
where
Z: Cat,
{
    fn foo(self) -> u64 {
        1
    }
}
