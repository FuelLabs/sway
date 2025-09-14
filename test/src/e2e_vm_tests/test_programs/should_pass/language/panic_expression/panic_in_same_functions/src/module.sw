library;

struct S {}

impl S {
    fn function() {
        panic "This is an error message.";
    }
}

fn function() {
    panic "This is an error message.";
}

pub fn module_main() {
    function();
    S::function();
}
