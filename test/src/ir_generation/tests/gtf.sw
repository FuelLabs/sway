script;

const SOME_TX_FIELD = 0x42;
const SOME_OTHER_TX_FIELD = 0x77;

fn main() {
    let field1 = __gtf::<u64>(1, SOME_TX_FIELD);
    let field2 = __gtf::<b256>(2, SOME_OTHER_TX_FIELD);
}

// ::check-ir::

// check: local u64 field1
// check: local b256 field2

// check: $(gtf1_index=$VAL) = const u64 1
// check: $(gtf1=$VAL) = gtf $gtf1_index, 66
// check: $(field1_var=$VAL) = get_local u64 field1
// check:  store $gtf1 to $field1_var

// check: $(gtf2_index=$VAL) = const u64 2
// check: $(gtf2=$VAL) = gtf $gtf2_index, 119
// check: $(gtf2_int_to_ptr=$VAL) = int_to_ptr $gtf2 to b256
// check: $(field2_var=$VAL) = get_local b256 field2
// check: store $gtf2_int_to_ptr to $field2_var

// ::check-asm::

// regex: REG=.r\d+\b

// check: gtf  $REG $$one i66

// check: lw   $(two=$REG) data_0
// check: gtf  $(b256_ptr=$REG) $two i119
// check: mcpi $REG $b256_ptr i32

// check: data_0 .word 2
