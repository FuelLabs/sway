script;

fn main() {
}

// ::check-ir::

// check: fn main() -> ()
// check: entry():

// ::check-asm::
// The data section setup:
// check: move $$$$locbase $$sp
// check: ret  $$zero
// nextln: .data
// not: data_
