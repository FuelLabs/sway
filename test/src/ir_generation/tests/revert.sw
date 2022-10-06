script;

fn revert(code: u64) {
    __revert(code);
}

fn main() {
    revert(42);
}

// ::check-ir::

// check: script {

// check: fn main() -> ()
// nextln: entry:
// nextln: $(value=$VAL) = const u64 42
// nextln: call revert_0($value)

// check: fn revert_0($(foo=$ID)
// nextln: entry:
// nextln: revert $foo
// nextln: } 
