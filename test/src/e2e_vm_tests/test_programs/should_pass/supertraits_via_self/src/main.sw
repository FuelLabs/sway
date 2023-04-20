contract;

struct S {}

trait MySuperTrait {
    fn foo();
}

trait MyTrait : MySuperTrait {
    fn bar();
} {
    // supertrait's methods are accessible in the default-implemented methods block
    fn baz() {
        Self::foo()
    }
}

impl MySuperTrait for S {
    fn foo() { }
}

impl MyTrait for S {
    // supertrait's methods are accessible in contract methods' bodies
    fn bar() { Self::foo() }
}
