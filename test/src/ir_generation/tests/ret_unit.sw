script;

fn f() {}

fn main() {
    f()
}

// ::check-ir::

// check: script {
// check: fn main() -> ()
// check: entry():
// check: call f_0()
// check: $(ret_v=$VAL) = const unit ()
// check: ret () $ret_v

// check: fn f_0() -> ()

// ::check-asm::
// check: ret  $$zero
