
library;

trait Cat {
    fn speak(self) -> u64;
}

struct S<T> {
    x: T,
}

impl<Z> S<S<Z>>
where
Z: Cat,
{
    fn foo(self) -> u64 {
        1
    }
}

impl S<S<u32>> {
    fn foo(self) -> u64 {
        1
    }
}

impl Cat for u32 {
    fn speak(self) -> u64 {
        1
    }
}
