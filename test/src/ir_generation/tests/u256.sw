script;

fn main() -> u64 {
    let a = 0u256 + 1u256;
    let b = 1u256 - 0u256;
    // c = NOT
    let d = 1u256 | 0u256;
    let e = 1u256 ^ 0u256;
    let f = 1u256 & 0u256;
    let g = 1u256 << 1;
    let h = 1u256 >> 1;

    let i = 2u256 * 2u256;
    let j = 2u256 / 2u256;

    let k = 2u256 == 2u256;
    let l = 2u256 < 2u256;
    let m = 2u256 > 2u256;

    let n = 2u256 <= 2u256;
    let o = 2u256 >= 2u256;
    0
}

// ::check-ir::
// check: v0 = const u256 0
// check: v1 = const u256 1
// check: v2 = call add_0(v0, v1)
// check: v3 = get_local ptr u256, a
// check: store v2 to v3

// check: entry(self: u256, other: u256)

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