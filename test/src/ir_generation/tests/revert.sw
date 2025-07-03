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
// check: entry():
// check: $(value=$VAL) = const u64 42
// check: call revert_0($value)

// check: fn revert_0($(foo=$ID)
// check: entry($foo: u64):
// checj: revert $foo
// check: } 
