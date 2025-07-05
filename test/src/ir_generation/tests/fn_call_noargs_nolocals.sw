// This is more to test the ASM generation rather than IR.

script;

fn f() {
    g()
}

fn g() {
}

fn main() {
    f();
}

// ::check-ir::

// check: fn main() -> ()

// ::check-asm::

// regex: IMM=i\d+
// regex: REG=\$r\d+

// Call a function:
//
// check: jal  $$$$reta $$pc $IMM

// Function calls other function, ignores result, returns unit/$zero.
//
// check: move $(reta_bk=$REG) $$$$reta
// check: jal  $$$$reta $$pc $IMM
// check: move $$$$retv $$zero
// check: move $$$$reta $reta_bk
// check: jal  $$zero $$$$reta i0

// Function returns unit.
//
// check: move $$$$retv $$zero
// check: jal  $$zero $$$$reta i0
