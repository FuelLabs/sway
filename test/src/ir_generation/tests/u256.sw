// optimisation-inline
script;

use core::ops::*;

fn main() -> u256 {
    let l = 1u256;
    let r = 1u256;
    l + r
}


// ::check-ir::
// check: const

// ::check-ir-optimized::
// pass: o1
// check: const

// ::check-asm::
// regex: REG=\$r\d+
// check: wqop $(a=$REG) $(b=$REG) $(c=$REG) i32 
// check: wqop $(d=$REG) $(e=$REG) $(f=$REG) i33 

// check: wqop $(d=$REG) $(e=$REG) $(f=$REG) i35
// check: wqop $(d=$REG) $(e=$REG) $(f=$REG) i36
// check: wqop $(d=$REG) $(e=$REG) $(f=$REG) i37

// check: wqml $(d=$REG) $(e=$REG) $(f=$REG) i48
// check: wqdv $(d=$REG) $(e=$REG) $(f=$REG) i32

// check: wqcm $(d=$REG) $(e=$REG) $(f=$REG) i32
// check: wqcm $(d=$REG) $(e=$REG) $(f=$REG) i34
// check: wqcm $(d=$REG) $(e=$REG) $(f=$REG) i35