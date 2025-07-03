// target-fuelvm

script;

fn main() {
    let recipient = 0x0000000000000000000000000000000000000000000000000000000000000000;
    let data = 5;
    let coins = 8;
    __smo(recipient, data, coins);
}

// ::check-ir::

// Match the first one where data is initialised.
// check: $(recip_ptr=$VAL) = get_local __ptr b256, __const

// Match the second one where we read data as a mem_copy_val later on
// check: $(data_ptr=$VAL) = get_local __ptr u64, data

// check: $(temp_ptr=$VAL) = get_local __ptr { u64, u64 }, $(=__anon_\d+)

// check: $(idx_0=$VAL) = const u64 0
// check: $(field_0_ptr=$VAL) = get_elem_ptr $temp_ptr, __ptr u64, $idx_0
// check: $(zero=$VAL) = const u64 0
// check: store $zero to $field_0_ptr

// check: $(idx_1=$VAL) = const u64 1
// check: $(field_1_ptr=$VAL) = get_elem_ptr $temp_ptr, __ptr u64, $idx_1
// check: mem_copy_val $field_1_ptr, $data_ptr

// check: $(coins_ptr=$VAL) = get_local __ptr u64, coins
// check: $(coins=$VAL) = load $coins_ptr
// check: $(sixtn=$VAL) = const u64 16
// check: smo $recip_ptr, $temp_ptr, $sixtn, $coins

// ::check-asm::

// regex: REG=\$r\d+

// check: smo  $REG $$$$locbase $REG $REG
