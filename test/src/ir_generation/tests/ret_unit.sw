script;

fn f() {}

fn main() {
    f()
}

// ::check-ir::
// regex: VAL=v\d+

// check: script {
// check: fn main() -> ()
// check: entry:
// check: call anon_0()
// check: $(ret_v=$VAL) = const unit ()
// check: ret () $ret_v

// check: fn anon_0() -> ()

// ::check-asm::
// check: ret  $$zero
