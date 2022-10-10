script;

struct A {}

impl A {
    fn generic<T>(self, x: T) -> T { x }
}

fn foo() -> bool {
    A {}.generic::<bool>(true)
}

fn main() {
    let _ = foo();
}
