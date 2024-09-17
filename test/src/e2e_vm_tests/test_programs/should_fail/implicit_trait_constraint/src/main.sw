script;
trait T2 {}
trait T1: T2 {
    fn new() -> Self;
}

struct S {}
impl T2 for S {}
impl T1 for S {
    fn new() -> Self {
        S {}
    }
}

fn bar<T>() -> T
where
 T: T1,
{
    T::new()
}

fn foo<T>() -> T
where
 T: T2,
{
    bar()
}

fn main() -> u64 {
    let _:S = foo();
    42
}
