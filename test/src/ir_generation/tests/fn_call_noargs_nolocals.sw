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
// check: movi $$$$reta $IMM
// check: ji   $IMM

// Function calls other function, ignores result, returns unit/$zero.
//
// check: move $(reta_bk=$REG) $$$$reta
// check: movi $$$$reta $IMM
// check: ji   $IMM
// check: move $$$$retv $$zero
// check: move $$$$reta $reta_bk
// check: jmp $$$$reta

// Function returns unit.
//
// check: move $REG $$$$reta
// check: move $$$$retv $$zero
// check: move $$$$reta $REG
// check: jmp $$$$reta
