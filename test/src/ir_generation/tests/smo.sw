script;

fn main() {
    let recipient = 0x0000000000000000000000000000000000000000000000000000000000000000;
    let data = 5; 
    let output_index = 4;
    let coins = 8;
    __smo(recipient, data, output_index, coins);
}

// ::check-ir::

// check: $(v10=$VAL) = get_ptr ptr { b256, u64, u64 } 
// check: $(v13=$VAL) = insert_value $v10, { b256, u64, u64 }, $VAL, 0
// check: $(v15=$VAL) = insert_value $v13, { b256, u64, u64 }, $VAL, 1
// check: $(v16=$VAL) = insert_value $v15, { b256, u64, u64 }, $VAL, 2
// check: $(v17=$VAL) = get_ptr ptr u64
// check: $(v18=$VAL) = load ptr $v17
// check: $(v19=$VAL) = get_ptr ptr u64
// check: $(v20=$VAL) = load ptr $v19
// check: $(v21=$VAL) = const u64 16
// check: smo $v16, $v21, $v18, $v20

// ::check-asm::

// regex: REG=\$r\d+

// check: smo  $REG $REG $REG $REG
