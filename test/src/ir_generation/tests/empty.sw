script;

fn main() {
}

// ::check-ir::

// check: fn main() -> ()
// check: entry:

// ::check-asm::
// The data section setup:
// check: ret  $$zero
// nextln: .data:
// not: data_
