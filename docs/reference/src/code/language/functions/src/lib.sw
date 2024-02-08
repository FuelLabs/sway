library;

mod explicit;
mod implicit;

// ANCHOR: definition
fn my_function(my_parameter: u64 /* ... */ ) -> u64 {
    // function code
    42
}
// ANCHOR_END: definition
// ANCHOR: equals
fn equals(first_parameter: u64, second_parameter: u64) -> bool {
    first_parameter == second_parameter
}
// ANCHOR_END: equals
fn usage() {
    // ANCHOR: usage
    let result_one = equals(5, 5); // evaluates to `true`
    let result_two = equals(5, 6); // evaluates to `false`
    // ANCHOR_END: usage
}

// ANCHOR: struct_definition
struct Foo {
    bar: u64,
}
// ANCHOR_END: struct_definition
// ANCHOR: method_impl
impl Foo {
    // refer to `bar`
    fn add_number(self, number: u64) -> u64 {
        self.bar + number
    }

    // mutate `bar`
    fn increment(ref mut self, number: u64) {
        self.bar += number;
    }
}
// ANCHOR_END: method_impl
fn method_usage() {
    // ANCHOR: method_usage
    let mut foo = Foo { bar: 42 };
    let result = foo.add_number(5); // evaluates to `47`
    foo.increment(5); // `bar` inside `foo` has been changed from 42 to 47
    // ANCHOR_END: method_usage
}

// ANCHOR: associated_impl
impl Foo {
    // this is an associated function because it does not take `self` as a parameter
    // it is also a constructor because it instantiates
    // and returns a new instance of `Foo`
    fn new(number: u64) -> Self {
        Self { bar: number }
    }
}
// ANCHOR_END: associated_impl
fn associated_usage() {
    // ANCHOR: associated_usage
    let foo = Foo::new(42);
    // ANCHOR_END: associated_usage
}
