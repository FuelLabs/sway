script;

fn get_global_gas() -> u64 {
    // This is just reading the $ggas register.  Need to make sure that's what the codegen does.
    asm() {
        ggas
    }
}

fn main() -> u64 {
    get_global_gas();
    asm(r1) {
        bhei r1;
        r1: u64
    }
}

// ::check-ir::

// check:  $(res=$VAL) = asm(r1) -> u64 r1
// nextln:     bhei   r1
// nextln: }
// check:  ret u64 $res

// check: $(gg=$VAL) = asm() -> u64 ggas
// check: ret u64 $gg

// ::check-asm::

// regex: REG=\$r\d+
// regex: IMM=i\d+

// Call get_global_gas() and then return the result of the bhei instruction.
// check: jal  $$$$reta $$pc $IMM
// check: bhei $(ret_val=$REG)
// check: ret  $ret_val

// The get_global_gas() function:
// check: move $$$$retv $$ggas
// check: jal  $$zero $$$$reta i0
