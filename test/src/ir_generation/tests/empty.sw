script;

fn main() {
}

// ::check-ir::

// check: fn main() -> ()
// check: entry():

// ::check-asm::
// The data section setup:
// check: move $$$$locbase $$sp
// not: cfei i0
// not: cfsi i0
// check: ret  $$zero
// nextln: .data
// not: data_
