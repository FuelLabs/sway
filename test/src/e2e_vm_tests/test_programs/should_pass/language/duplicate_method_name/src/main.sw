script;

trait Cat {
    fn speak(self) -> u64;
}

struct S<T> {
    x: T,
}

impl<Z> S<Z>
where
Z: Cat,
{
    fn foo(self) -> u64 {
        // self.x.speak()
        1 
    }
}

impl S<u32> {
    fn foo(self) -> u64 {
        // self.x.speak()
        1 
    }
}

fn main() {
    let s = S::<u32> { x: 1 };
    s.foo();
}