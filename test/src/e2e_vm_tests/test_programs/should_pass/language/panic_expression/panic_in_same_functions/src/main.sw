// This test proves that https://github.com/FuelLabs/sway/issues/7386 is fixed.
script;

mod module;

struct S {}

impl S {
    fn function() {
        panic "This is an error message.";
    }
}

fn function() {
    panic "This is an error message.";
}

fn main() {
    function();
    S::function();
    module::module_main();
}
