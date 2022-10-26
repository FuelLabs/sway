contract;

struct A {}

impl A {
    fn generic<T>(self, x: T) -> T { x }
}

fn foo() -> bool {
    A {}.generic::<u8>(true)
}
