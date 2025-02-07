
library;

trait Cat {
    fn speak(self) -> u64;
}

struct S<T, Y> {
    x: T,
    y: Y,
}

impl S<u32, u64> {
    fn foo(self) -> u64 {
        1
    }
}

impl<Z, Y> S<Z, Y>
where
Z: Cat, Y: Cat
{
    fn foo(self) -> u64 {
        1
    }
}

impl Cat for u32 {
    fn speak(self) -> u64 {
        1
    }
}

impl Cat for u64 {
    fn speak(self) -> u64 {
        1
    }
}