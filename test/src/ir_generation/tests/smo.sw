// target-fuelvm

script;

fn main() {
    let recipient = 0x0000000000000000000000000000000000000000000000000000000000000000;
    let data = 5;
    let output_index = 4;
    let coins = 8;
    __smo(recipient, data, output_index, coins);
}

// ::check-ir::

// Match the first one where data is initialised.
// check: get_local ptr u64, data

// Match the second one where we read it back.
// check: $(data_ptr=$VAL) = get_local ptr u64, data

// check: $(temp_ptr=$VAL) = get_local ptr { b256, u64, u64 }, $(=__anon_\d+)

// check: $(recip_ptr=$VAL) = get_local ptr b256, recipient
// check: $(idx_0=$VAL) = const u64 0
// check: $(field_0_ptr=$VAL) = get_elem_ptr $temp_ptr, ptr b256, $idx_0
// check: mem_copy_val $field_0_ptr, $recip_ptr

// check: $(idx_1=$VAL) = const u64 1
// check: $(field_1_ptr=$VAL) = get_elem_ptr $temp_ptr, ptr u64, $idx_1
// check: $(zero=$VAL) = const u64 0
// check: store $zero to $field_1_ptr

// check: $(idx_2=$VAL) = const u64 2
// check: $(field_2_ptr=$VAL) = get_elem_ptr $temp_ptr, ptr u64, $idx_2
// check: mem_copy_val $field_2_ptr, $data_ptr

// check: $(oi_ptr=$VAL) = get_local ptr u64, output_index
// check: $(oi=$VAL) = load $oi_ptr
// check: $(coins_ptr=$VAL) = get_local ptr u64, coins
// check: $(coins=$VAL) = load $coins_ptr
// check: $(sixtn=$VAL) = const u64 16
// check: smo $temp_ptr, $sixtn, $oi, $coins

// ::check-asm::

// regex: REG=\$r\d+

// check: smo  $REG $REG $REG $REG
