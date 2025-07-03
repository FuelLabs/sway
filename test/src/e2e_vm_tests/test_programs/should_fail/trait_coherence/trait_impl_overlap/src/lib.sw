library;

// Define two traits
trait TraitA {
    fn do_something(self);
}

trait TraitB {
    fn do_something_else(self);
}

// Implement both traits for a concrete type
struct MyType {}

impl TraitA for MyType {
    fn do_something(self) {
    }
}

impl TraitB for MyType {
    fn do_something_else(self) {
    }
}

// Blanket implementation that may cause overlap
impl<T> TraitA for T {
    fn do_something(self) {
    }
}
